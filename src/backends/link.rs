use crate::config::Config;
use crate::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, PackageSpec, Result,
};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tera::{Context, Tera};
use tracing::{debug, info, warn};

pub struct LinkBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
    pub config: Arc<Config>,
    /// Declared secret-decryption providers (U38), beyond the built-in `age`/`sops`.
    pub secret_providers: Vec<crate::model::secret::SecretProvider>,
}

/// Where a `@target=` value lands on disk. The install path and the pre-sync confirmation
/// must answer this the same way, or the run confirms one destination and writes another.
pub fn resolve_target(target: &str) -> Result<PathBuf> {
    crate::core::account::Account::current()?.expand_home(target)
}

pub fn resolve_target_for_user(target: &str, user: &str) -> Result<PathBuf> {
    crate::core::account::Account::resolve(user)?.expand_home(target)
}

/// Where a declared source file lives on disk — the `link:` name, or a `dotfiles:` tree.
pub fn resolve_source(config: &Config, declared: &str) -> Result<PathBuf> {
    resolve_source_for_user(config, declared, None)
}

pub fn resolve_source_for_user(
    config: &Config,
    declared: &str,
    _user: Option<&str>,
) -> Result<PathBuf> {
    let path = PathBuf::from(declared);
    if path.is_absolute() {
        return Ok(path);
    }
    if declared == "~" || declared.starts_with("~/") {
        return resolve_target(declared);
    }
    Ok(config.config_root().join(path))
}

/// Whether a `link:` line reads its source file at all.
pub fn reads_its_source(opts: &crate::config::grammar::Options) -> bool {
    opts.one("content").is_none()
}

/// [`resolve_source`], refusing when the file is not on disk.
pub fn resolve_existing_source(config: &Config, declared: &str) -> Result<PathBuf> {
    resolve_existing_source_for_user(config, declared, None)
}

pub fn resolve_existing_source_for_user(
    config: &Config,
    declared: &str,
    user: Option<&str>,
) -> Result<PathBuf> {
    let resolved = resolve_source_for_user(config, declared, user)?;
    if resolved.exists() {
        return Ok(resolved);
    }
    Err(Error::Validation(format!(
        "`link:{}` names a file that is not there ({}). A relative source is read from your \
         config repo at {} — check the path, or make it absolute. Shall will not place a link \
         to nothing: it would exist, and nothing could open it.",
        declared,
        resolved.display(),
        config.config_root().display()
    )))
}

pub async fn chown_to_user(
    path: &Path,
    user: &str,
    executor: &CommandExecutor,
    follow: bool,
) -> Result<()> {
    let account = crate::core::account::Account::resolve(user)?;
    let owns = match std::fs::symlink_metadata(path) {
        Ok(_) => account.owns(path, follow)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(Error::from(error)),
    };
    if owns {
        return Ok(());
    }
    if executor.dry_run {
        crate::would!("Link: would chown {:?} to {}", path, user);
        return Ok(());
    }
    account.apply_owner(path, follow)?;
    info!("Link: chown {:?} to {}", path, user);
    Ok(())
}

async fn apply_declared_owner(
    path: &Path,
    spec: &PackageSpec,
    executor: &CommandExecutor,
) -> Result<()> {
    if let Some(owner) = spec
        .options
        .one("owner")
        .or_else(|| spec.options.one("user"))
    {
        let follow = spec.options.contains("content")
            || spec.options.contains("decrypt")
            || spec.options.one("template") == Some("true");
        chown_to_user(path, owner, executor, follow).await?;
    }
    Ok(())
}

pub async fn ensure_parent_dir_for_user(
    path: &Path,
    executor: &CommandExecutor,
    user: Option<&str>,
) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let mut missing = Vec::new();
    let mut current = parent;
    while !current.exists() {
        missing.push(current.to_path_buf());
        current = current
            .parent()
            .ok_or_else(|| Error::Validation(format!("cannot find a parent for {:?}", path)))?;
    }
    if executor.dry_run {
        for path in missing.iter().rev() {
            crate::would!("Link: would create parent {:?}", path);
        }
        return Ok(());
    }
    for path in missing.iter().rev() {
        tokio::fs::create_dir(path).await.map_err(Error::from)?;
        if let Some(user) = user {
            chown_to_user(path, user, executor, true).await?;
        }
    }
    Ok(())
}

/// Where the pre-existing file at `target` is kept while Shall owns that path. One function,
/// because the write path and the undo path must agree on the name or a restore looks for a
/// file nothing wrote.
pub fn backup_path(target: &Path) -> PathBuf {
    PathBuf::from(format!("{}.shall-backup", target.display()))
}

/// Copy a whole directory tree to `backup`, preserving its shape, before Shall replaces it.
///
/// A symlink inside the tree is recreated as a pointer rather than followed — following it is
/// how a link out of the tree drags in something huge, and how two links naming each other
/// recurse forever. The depth bound is the backstop for exactly that cycle on hosts where
/// recreating pointers is not possible and copying through them is the only remaining way to
/// keep their data.
fn backup_directory_tree<'a>(
    src: &'a Path,
    dst: &'a Path,
    depth: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        const MAX_DEPTH: usize = 64;
        if depth > MAX_DEPTH {
            // Exit 3, not 1: this is a refusal (U21), and the refusal hook exists to hear
            // about exactly this class of sentence.
            return Err(Error::Refused(format!(
                "refusing to back up {:?}: deeper than {MAX_DEPTH} levels, which usually means \
                 a symlink loop inside it",
                src
            )));
        }
        tokio::fs::create_dir_all(dst).await.map_err(Error::from)?;
        let mut rd = tokio::fs::read_dir(src).await.map_err(Error::from)?;
        while let Some(entry) = rd.next_entry().await.map_err(Error::from)? {
            let from = entry.path();
            let to = dst.join(entry.file_name());
            let ft = entry.file_type().await.map_err(Error::from)?;
            if ft.is_symlink() {
                let dest = tokio::fs::read_link(&from).await.map_err(Error::from)?;
                match recreate_link(&dest, &to).await {
                    Ok(()) => {}
                    Err(e) => {
                        // No privilege to recreate the pointer: keep the data by copying
                        // through it, bounded above so two pointers naming each other stop
                        // instead of spinning.
                        warn!(
                            "Link: could not preserve the pointer at {:?} ({e}); copying what \
                             it points at.",
                            from
                        );
                        backup_directory_tree(&from, &to, depth + 1).await?;
                    }
                }
            } else if ft.is_dir() {
                backup_directory_tree(&from, &to, depth + 1).await?;
            } else {
                tokio::fs::copy(&from, &to).await.map_err(Error::from)?;
            }
        }
        Ok(())
    })
}

#[cfg(unix)]
async fn recreate_link(dest: &Path, to: &Path) -> Result<()> {
    tokio::fs::symlink(dest, to).await.map_err(Error::from)
}

#[cfg(windows)]
async fn recreate_link(dest: &Path, to: &Path) -> Result<()> {
    // A Windows pointer must be created knowing whether it names a directory; the stored
    // string does not say, so probe what it resolves to and default to the file form when it
    // resolves to nothing.
    let dir = tokio::fs::metadata(dest)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false);
    match tokio::fs::symlink_dir(dest, to).await {
        Ok(()) => Ok(()),
        Err(_e) if !dir => tokio::fs::symlink_file(dest, to).await.map_err(Error::from),
        Err(e) => Err(Error::from(e)),
    }
}

/// Whether a `link:` line wants its pre-existing target preserved (T6). Backing up is the
/// default; `@backup=no` opts a single line out, stated where the exception is. A machine-wide
/// key was deliberately not added — restore-on-removal already kills the pile-up one would have
/// been for, so one mechanism answers the whole question.
pub fn wants_backup(spec: &PackageSpec) -> bool {
    !matches!(spec.options.one("backup"), Some("no") | Some("false"))
}

/// Refuse a decrypted secret whose destination is inside the config repo (T2).
///
/// The repo is git-tracked and `sync` commits it, so a plaintext written there is a plaintext
/// in history — and a secret in git history is a rotated secret, which is unrecoverable rather
/// than merely bad. The refusal names both paths, because "somewhere inside your repo" is not
/// something a reader can act on.
///
/// **Where it lands, not where it is written.** The comparison resolves the symlinks in the
/// destination's existing prefix, because a first install has no file to canonicalize and a
/// string comparison is what was there: `link:./token@target=/tmp/x/token,decrypt=age` with
/// `/tmp/x` a symlink into the repo passed, and the plaintext landed in a tree the next `sync`
/// commits. `resolves_within` fails closed if either side cannot be resolved, because a guard
/// that answers "outside" because it could not look has stopped guarding.
pub fn refuse_target_in_repo(config: &Config, resolved: &Path) -> Result<()> {
    let root = config.config_root();
    if !crate::core::validator::Validator::resolves_within(resolved, &root)? {
        return Ok(());
    }
    Err(Error::Refused(format!(
        "refusing to decrypt into {} — it is inside the config repo at {}, which git tracks \
         and `sync` commits. A secret that reaches git history has to be rotated, not deleted. \
         Point @target= outside the repo.",
        resolved.display(),
        root.display()
    )))
}

pub fn is_outside_home(resolved: &Path) -> bool {
    is_outside_home_for_user(resolved, None).unwrap_or(true)
}

/// Whether this destination is outside the account's home, and so needs the confirmation.
///
/// **The same resolution, for the same reason.** A destination written as `/tmp/x/.vimrc` is
/// outside the home as text and inside it on disk when `/tmp/x` is a symlink into it, and the
/// question this answers is the one a user is asked before Shall writes outside their home. An
/// answer that cannot see the symlink is an answer that skips the ask.
pub fn is_outside_home_for_user(resolved: &Path, user: Option<&str>) -> Result<bool> {
    let account = match user {
        Some(user) => crate::core::account::Account::resolve(user)?,
        None => crate::core::account::Account::current()?,
    };
    Ok(!crate::core::validator::Validator::resolves_within(
        resolved,
        account.home(),
    )?)
}

/// The argument list for a decrypt tool. `-i` takes the identity as its value, so it stays
/// in front of the terminator; only the source path goes behind it.
/// **`pub` for the integration tests' sake.** The link mirror must assert the exact decrypt argv
/// and terminator ordering without duplicating the command builder.
pub fn decrypt_argv(tool: &str, source: &Path, identity: Option<&Path>) -> Result<Vec<String>> {
    let mut args = vec!["--decrypt".to_string()];
    match tool {
        "age" => {
            let identity = identity.ok_or_else(|| {
                Error::Other(
                    "age decrypt needs an identity — set @identity=<path> or $SHALL_AGE_IDENTITY"
                        .into(),
                )
            })?;
            args.push("-i".to_string());
            args.push(identity.to_string_lossy().to_string());
        }
        "sops" => {}
        other => {
            return Err(Error::Other(format!(
                "unknown decrypt tool '{}' (use age or sops)",
                other
            )))
        }
    }
    crate::core::argv::push_names(&mut args, tool, [source.to_string_lossy().to_string()]);
    Ok(args)
}

impl LinkBackendCore {
    pub fn new(executor: CommandExecutor, config: Arc<Config>) -> Self {
        Self {
            executor,
            name: "link".to_string(),
            config,
            secret_providers: Vec::new(),
        }
    }

    /// Attach declared secret providers (U38), loaded from `adapters/secret.toml`.
    pub fn with_secret_providers(
        mut self,
        providers: Vec<crate::model::secret::SecretProvider>,
    ) -> Self {
        self.secret_providers = providers;
        self
    }

    /// The conventional directory for template secret files: `~/.config/shall/secrets/`.
    ///
    /// A `secret("github-token")` in a template resolves against a file at this path.
    /// The directory is created on first use rather than at startup — most machines never
    /// place a template secret, and a startup `mkdir` would be noise.
    fn secrets_dir(&self) -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".config").join("shall").join("secrets"))
    }

    /// Find the encrypted file backing a named template secret.
    ///
    /// Searches `~/.config/shall/secrets/<name>.age`, then `<name>` (for providers
    /// that do not use the `.age` extension). Returns `None` when neither exists — the
    /// caller decides whether that is an error or a skip.
    fn find_secret_file(&self, name: &str) -> Option<PathBuf> {
        let dir = self.secrets_dir()?;
        let with_ext = dir.join(format!("{}.age", name));
        if with_ext.exists() {
            return Some(with_ext);
        }
        let bare = dir.join(name);
        if bare.exists() {
            return Some(bare);
        }
        None
    }

    /// Decrypt a named template secret, returning its plaintext.
    ///
    /// The secret file is located via [`find_secret_file`] and decrypted with the default
    /// provider (`age`). Custom providers registered in `adapters/secret.toml` are tried
    /// in order when `age` is not available.
    async fn resolve_secret(&self, name: &str) -> Result<String> {
        let source = self.find_secret_file(name).ok_or_else(|| {
            let dir = self
                .secrets_dir()
                .map(|d| d.display().to_string())
                .unwrap_or_else(|| "~/.config/shall/secrets".to_string());
            Error::Other(format!(
                "template references `${{secret:{}}}` but no encrypted file exists at {}/{}.age \
                 (or {}/{}). Create the file or remove the ${{secret:{}}} reference.",
                name, dir, name, dir, name, name
            ))
        })?;
        // Build a minimal PackageSpec for the decrypt call — the spec carries the identity
        // option, which `decrypt_secret` reads.
        let spec = PackageSpec {
            name: source.to_string_lossy().to_string(),
            backend: "link".into(),
            options: crate::config::grammar::Options::default(),
            requires: vec![],
            present: true,
        };
        match self.decrypt_secret("age", &source, &spec).await? {
            Some(plaintext) => Ok(plaintext),
            None => Err(Error::Other(format!(
                "${{secret:{}}}` requires a physical touch but this is an unattended run",
                name
            ))),
        }
    }

    /// Pre-fetch every secret a template references, returning a context map.
    ///
    /// Scans `text` for `${secret:name}` references, decrypts each referenced secret, and
    /// returns the name→plaintext map. A secret that cannot be resolved is an `Err`
    /// naming the missing name — the sync stops rather than silently rendering an
    /// unresolved `${secret:name}` reference.
    pub async fn resolve_template_secrets(&self, text: &str) -> Result<SecretContext> {
        let names = secrets_in_template(text);
        if names.is_empty() {
            return Ok(SecretContext::new());
        }
        let mut ctx = SecretContext::new();
        for name in &names {
            if ctx.contains_key(name) {
                continue;
            }
            let plaintext = self.resolve_secret(name).await?;
            ctx.insert(name.clone(), plaintext);
        }
        Ok(ctx)
    }

    /// Resolve the age identity file: explicit `@identity=`, else `$SHALL_AGE_IDENTITY`,
    /// else the conventional `~/.config/shall/age.key`.
    fn age_identity(&self, spec: &PackageSpec) -> Option<PathBuf> {
        if let Some(id) = spec.options.one("identity") {
            return Some(PathBuf::from(id));
        }
        if let Ok(id) = std::env::var("SHALL_AGE_IDENTITY") {
            return Some(PathBuf::from(id));
        }
        dirs::home_dir().map(|h| h.join(".config").join("shall").join("age.key"))
    }

    /// Decrypt an encrypted source file to plaintext by shelling out to the `age` or `sops`
    /// binary. Shall stays true to its "manager of managers" model: it orchestrates the
    /// tool the user already trusts rather than embedding crypto. stdout is captured raw
    /// (never trimmed) so key material survives byte-for-byte.
    /// Decrypt `source`, or return `Ok(None)` when an unattended run skips a touch-required
    /// line (T4).
    ///
    /// `Ok(None)` is a deliberate skip, not a failure: T4 says a `watch` tick does not block the
    /// whole reconcile waiting for a physical touch nobody will give. `Ok(Some)` is the
    /// plaintext; an `Err` is a real failure — including a decrypt that hung past the timeout
    /// (T3), which is the touch-required case reached at a terminal rather than under `watch`.
    pub(crate) async fn decrypt_secret(
        &self,
        tool: &str,
        source: &Path,
        spec: &PackageSpec,
    ) -> Result<Option<String>> {
        use tokio::process::Command;
        let identity = self.age_identity(spec);

        // Is this a hardware/interactive identity? Read the identity file and look for an age
        // plugin marker. A file we cannot read is treated as not-a-plugin: the decrypt below
        // will fail with age's own error if the identity is genuinely bad, which is the honest
        // report.
        let plugin = identity
            .as_deref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|c| crate::model::secret::plugin_of(&c));

        // T4: an unattended `watch` tick does not attempt a touch-required line.
        if self.config.unattended {
            if let Some(plugin) = &plugin {
                warn!(
                    "{}",
                    crate::model::secret::watch_skip_message(source, plugin)
                );
                return Ok(None);
            }
        }

        // `age`/`sops` are built in (age carries the hardware handling above); anything else is a
        // declared provider (U38), which plugs into this same plaintext-handling path — captured
        // from stdout, bounded by the timeout, restricted before it is written. A provider is
        // trusted only because it promised stdout-only and the ledger approved its file.
        let (program, args) = if tool == "age" || tool == "sops" {
            (
                tool.to_string(),
                decrypt_argv(tool, source, identity.as_deref())?,
            )
        } else {
            let provider = self
                .secret_providers
                .iter()
                .find(|p| p.name == tool)
                .ok_or_else(|| {
                    let mut known = vec!["age".to_string(), "sops".to_string()];
                    known.extend(self.secret_providers.iter().map(|p| p.name.clone()));
                    Error::Other(format!(
                        "unknown decrypt tool '{}' — known: {}. Add a `[[secret]]` row to \
                         `adapters/secret.toml` for another provider.",
                        tool,
                        known.join(", ")
                    ))
                })?;
            let id = identity.as_deref().map(|p| p.to_string_lossy().to_string());
            provider
                .command(&source.to_string_lossy(), id.as_deref())
                .ok_or_else(|| {
                    Error::Other(format!("the `{}` secret provider has no command", tool))
                })?
        };
        let mut cmd = Command::new(&program);
        cmd.args(&args);
        // T3: a decrypt that does not complete is waiting on a prompt nobody will answer. Bound
        // it, and on timeout name the token and the identity rather than leaving the process
        // (and this sync) hung forever.
        //
        // **And the process really does stop now.** Dropping the future that awaits a child does
        // not kill it, so this bound used to free the sync and leave `gpg` running against a
        // prompt for as long as the machine was up — the one thing the comment above promised it
        // would not do. `supervised_output` owns the child, so the timeout below reaches it.
        let output = match tokio::time::timeout(
            crate::model::secret::DECRYPT_TIMEOUT,
            crate::core::supervise::supervised_output(cmd, &program, false),
        )
        .await
        {
            Ok(result) => result.map_err(|e| {
                Error::Other(format!(
                    "could not launch '{}' to decrypt {:?}: {} — is it installed and on PATH?",
                    tool, source, e
                ))
            })?,
            Err(_) => {
                return Err(Error::Other(crate::model::secret::token_timeout_message(
                    source,
                    identity.as_deref().unwrap_or(Path::new("(none)")),
                    plugin.as_deref(),
                )));
            }
        };
        if !output.status.success() {
            return Err(Error::Other(format!(
                "{} failed to decrypt {:?}: {}",
                tool,
                source,
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        String::from_utf8(output.stdout).map(Some).map_err(|e| {
            Error::Other(format!(
                "decrypted content of {:?} is not valid UTF-8: {}",
                source, e
            ))
        })
    }
}

/// The facts a `link:` template renders against, as Tera sees them. One constructor for
/// the installer and the checker, so the two cannot disagree about what `{{ HOME }}`
/// means: a template `check` calls unverifiable while `sync` places is a drift report
/// that never settles. Keys are the existing uppercase vocabulary (`OS`, `ARCH`, `USER`,
/// `HOSTNAME`) plus `HOME` and `FAMILY`; `aliases` rides along as before. A fact the
/// machine could not detect is absent rather than `"unknown"`, so a template that needs
/// it fails naming the variable instead of writing a path with `unknown` in it — except
/// `USER`, which keeps its long-standing `"unknown"` fallback so existing templates do
/// not start failing where they used to render.
/// Build the template variable context (II.8, II.21).
///
/// `$user` and `$home` are not hardcoded — they are resolved from the process environment,
/// with a fallback for hosts that do not set `$USER`. A `link:` whose `@template=true` sees
/// the same variables as the program's variables output.
///
/// When `user_home` is provided (U71 — per-user `@target=` with `@user=NAME`), it overrides
/// the HOME and USER variables for that particular link.
pub fn template_context(
    facts: &crate::config::parser::HostFacts,
    config: &Config,
    user_home: Option<(&str, &str)>,
) -> Context {
    let mut context = Context::new();
    context.insert("OS".to_string(), &facts.os);
    context.insert("ARCH".to_string(), &facts.arch);
    context.insert("HOSTNAME".to_string(), &facts.host);
    context.insert("FAMILY".to_string(), &facts.family);
    if let Some((user, home)) = user_home {
        context.insert("USER".to_string(), user);
        context.insert("HOME".to_string(), home);
    } else {
        context.insert(
            "USER".to_string(),
            facts.user.as_deref().unwrap_or("unknown"),
        );
        if let Some(home) = &facts.home {
            context.insert("HOME".to_string(), home);
        }
    }
    context.insert("aliases".to_string(), &config.aliases);
    context
}

/// Every `secret("name")` call in a template body, in the order they appear.
///
/// The pattern is `${secret:name}` — the same syntax in `content:`, `link:` templates,
/// and `nixos:` config text. Names are returned in order so a missing secret can name
/// its position.
pub fn secrets_in_template(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"\$\{secret:([^}]+)\}"#)
        .expect("the secret-substitution regex is valid");
    re.captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// A map of secret name to its decrypted plaintext, ready for template context injection.
pub type SecretContext = std::collections::HashMap<String, String>;

/// Render template `text` from `source` against the shared context. Pure apart from its
/// arguments: the installer reads the file through the executor (dry-run VFS aware) and the
/// checker reads it off the disk, and both render here.
///
/// `${secret:name}` references must be resolved **before** calling this function — the
/// rendering is Tera, which knows nothing about secrets. Pre-process with
/// [`substitute_secrets`] (from the nixos module) to replace them with plaintext.
///
/// When `user` is provided (U71), $HOME and $user in the template resolve to that user's
/// home directory and name respectively.
pub fn render_template_text(
    source: &Path,
    text: &str,
    facts: &crate::config::parser::HostFacts,
    config: &Config,
    user: Option<&str>,
) -> Result<String> {
    let user_home = user
        .map(|name| {
            let account = crate::core::account::Account::resolve(name)?;
            let home = account.home().to_str().ok_or_else(|| {
                Error::Other("the selected account home is not valid UTF-8".into())
            })?;
            Ok::<_, Error>((name, home.to_string()))
        })
        .transpose()?;
    let user_home_ref = user_home
        .as_ref()
        .map(|(name, home)| (*name, home.as_str()));
    let mut tera = Tera::default();
    tera.add_raw_template("config", text)
        .map_err(|e| Error::Other(format!("Tera Parse Error in {:?}: {}", source, e)))?;
    tera.render("config", &template_context(facts, config, user_home_ref))
        .map_err(|e| Error::Other(format!("Tera Render Error in {:?}: {}", source, e)))
}

impl LinkBackendCore {
    /// Render the template at `source_path` against this machine's facts. Reads through
    /// the executor rather than off the disk, so a dry run renders the file this run
    /// would read rather than failing on one that is not there yet.
    ///
    /// `${secret:name}` references are resolved before Tera rendering, so the same
    /// syntax works in `content:`, `link:` templates, and `nixos:` config text.
    async fn render_template(&self, source_path: &Path, user: Option<&str>) -> Result<String> {
        let content = self.executor.read_file(source_path).await?;
        let pre_rendered = if content.contains("${secret:") {
            let secrets = self.resolve_template_secrets(&content).await?;
            crate::backends::nixos::substitute_secrets(&content, &secrets)?
        } else {
            content
        };
        render_template_text(
            source_path,
            &pre_rendered,
            &crate::config::parser::HostFacts::current(),
            &self.config,
            user,
        )
    }

    /// Write `desired` content to `target`, idempotently. If the target already holds
    /// exactly this content it is left untouched (no backup, no write); otherwise any
    /// pre-existing file is backed up (once) before the managed content is written.
    /// Shared by inline-content and rendered-template modes.
    async fn apply_managed_content(
        &self,
        target: &Path,
        desired: &str,
        backup: bool,
    ) -> Result<()> {
        if let Ok(existing) = self.executor.read_file(target).await {
            if existing == desired {
                debug!("Link: {:?} is already up-to-date.", target);
                return Ok(());
            }
        }
        if backup {
            self.backup_once(target).await?;
        }
        info!("Link: Writing managed file {:?}", target);
        self.executor.write_atomic(target, desired).await?;
        Ok(())
    }

    /// Preserve a pre-existing, unmanaged file or directory before Shall overwrites or replaces
    /// it — exactly once, as `<target>.shall-backup`. So the user is never silently robbed of
    /// a config they already had. Symlinks (mere pointers) are skipped, and an existing backup
    /// is never clobbered, so the true original survives even across repeated syncs. Honors
    /// dry-run (previews instead of copying).
    async fn backup_once(&self, target: &Path) -> Result<()> {
        // A symlink is just a pointer; the real data lives elsewhere and is untouched.
        if target.is_symlink() {
            return Ok(());
        }
        let meta = match tokio::fs::symlink_metadata(target).await {
            Ok(m) => m,
            Err(_) => return Ok(()), // nothing there to preserve
        };
        let backup = backup_path(target);
        if let Ok(backup_meta) = tokio::fs::symlink_metadata(&backup).await {
            // The original was already preserved on an earlier run — but only counts as
            // preserved if it is the same *kind* of thing. A file backup beside what is now a
            // directory (or the reverse) does not cover the target, and treating it as covered
            // would let the replacement below destroy data nothing copied.
            if backup_meta.is_dir() == meta.is_dir() {
                return Ok(());
            }
            return Err(Error::Other(format!(
                "{:?} holds a {} while {:?} holds a {} from an earlier run — one of them is \
                 not what Shall expects. Move one aside yourself; Shall will not overwrite \
                 either.",
                target,
                if meta.is_dir() { "directory" } else { "file" },
                backup,
                if backup_meta.is_dir() {
                    "directory"
                } else {
                    "file"
                },
            )));
        }
        if self.executor.dry_run {
            crate::would!(
                "Link: would back up existing {} {:?} to {:?} before writing the managed version.",
                if meta.is_dir() { "directory" } else { "file" },
                target,
                backup
            );
            return Ok(());
        }
        if meta.is_dir() {
            // A directory at the target is the user's own tree, and the most common real
            // shape (~/.config/nvim). Not backing it up here used to mean the replacement
            // below deleted it whole — the promise this function exists for stopped exactly
            // where the data was biggest.
            backup_directory_tree(target, &backup, 0).await?;
            info!(
                "Link: Existing directory {:?} was backed up to {:?} before applying the managed version.",
                target, backup
            );
            return Ok(());
        }
        tokio::fs::copy(target, &backup)
            .await
            .map_err(Error::from)?;
        info!(
            "Link: Existing {:?} was backed up to {:?} before applying the managed version.",
            target, backup
        );
        Ok(())
    }
}

/// Whether Windows refused a symlink because the process may not create one.
///
/// `CreateSymbolicLinkW` fails with `ERROR_PRIVILEGE_NOT_HELD` (1314) unless the process is
/// elevated or Developer Mode is on. The executor renders io errors to strings before this
/// layer sees them, so the code is matched in the one rendering Rust guarantees for an OS
/// error: a trailing `(os error N)`.
#[cfg(target_os = "windows")]
fn is_missing_symlink_privilege(e: &Error) -> bool {
    matches!(e, Error::Io(msg) if msg.contains("os error 1314"))
}

/// Said when a `link:` becomes a copy, because the two differ in a way the user will meet
/// later: edits to the source stop appearing at the destination until the next sync. Names
/// the privilege and how to get it, since "fell back to a copy" is not something a reader
/// can act on.
#[cfg(target_os = "windows")]
fn copy_fallback_message(source: &Path, target: &Path) -> String {
    format!(
        "Link: placed {:?} as a COPY of {:?}, not a link — creating symlinks needs Developer \
         Mode or an elevated shell. Edits to the source will not appear at the destination \
         until the next sync. Settings > For developers > Developer Mode turns this into a \
         real link.",
        target, source
    )
}

#[async_trait]
impl BackendCore for LinkBackendCore {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        true
    }
    fn probes(&self) -> Vec<String> {
        Vec::new()
    }
    fn needs_root(&self) -> bool {
        false
    }
}

/// Filesystem objects have no native transitive deps.
#[async_trait]
impl MetadataProvider for LinkBackendCore {
    async fn get_dependencies(&self, _name: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct LinkInstallable {
    pub core: Arc<LinkBackendCore>,
}

#[async_trait]
impl Installable for LinkInstallable {
    /// Must go through `executor.symlink()` rather than `tokio::fs` directly: only the
    /// executor records into the dry-run VFS, so a direct call would touch the real
    /// filesystem during a dry run.
    async fn install(&self, specs: &[PackageSpec], _: bool) -> Result<()> {
        for spec in specs {
            let target_str = spec
                .options
                .one("target")
                .ok_or_else(|| Error::Other("Link requires @target".into()))?;

            // U71: when @user=NAME is present, resolve ~/ to that user's home.
            let user = spec.options.one("user");
            let target_path = match user {
                Some(u) => resolve_target_for_user(target_str, u)?,
                None => resolve_target(target_str)?,
            };
            if let Some(owner) = spec.options.one("owner") {
                crate::core::account::Account::resolve(owner)?;
            }

            // U71: create parent directories if auto_create_parent_dirs is enabled.
            if self.core.config.link.auto_create_parent_dirs {
                ensure_parent_dir_for_user(&target_path, &self.core.executor, user).await?;
            }
            let backup = wants_backup(spec);

            // Mode A: Inline content declared directly (no separate source file).
            // Supports `${secret:name}` substitution for secret composition.
            if let Some(content) = spec.options.one("content") {
                let resolved = if content.contains("${secret:") {
                    let secrets = self.core.resolve_template_secrets(content).await?;
                    crate::backends::nixos::substitute_secrets(content, &secrets)?
                } else {
                    content.to_string()
                };
                self.core
                    .apply_managed_content(&target_path, &resolved, backup)
                    .await?;
                apply_declared_owner(&target_path, spec, &self.core.executor).await?;
                continue;
            }

            // Resolved, not stored verbatim. The symlink written below keeps whatever string it
            // is given and resolves it against the *link's* directory, so a relative source is
            // a dangling link at a path nobody in the config repo named (B0b). Refused rather
            // than resolved when the file is not there, because the three modes below all read
            // it and a link to nothing is the failure this whole path exists to prevent.
            let source = resolve_existing_source(&self.core.config, &spec.name)?;

            // Mode D: Secret — decrypt the source with age/sops and place the plaintext.
            if let Some(tool) = spec.options.one("decrypt") {
                // T2, before anything is decrypted: the config root is a git repo, and a
                // plaintext written inside it is committed by the next sync. A secret in git
                // history is a rotated secret, so this is a refusal rather than a warning.
                refuse_target_in_repo(&self.core.config, &target_path)?;
                if self.core.executor.dry_run {
                    crate::would!(
                        "Link: would decrypt {:?} with {} and write to {:?}",
                        source,
                        tool,
                        target_path
                    );
                    apply_declared_owner(&target_path, spec, &self.core.executor).await?;
                    continue;
                }
                // `None` is T4's deliberate skip (an unattended tick met a touch-required key);
                // decrypt_secret already said so. Move to the next line rather than failing.
                let Some(plaintext) = self.core.decrypt_secret(tool, &source, spec).await? else {
                    continue;
                };
                // T1: no backup. `backup_once` exists so a user is not silently robbed of a
                // config file they hand-wrote; a secret Shall decrypted a moment ago is not
                // that, and the copy would sit beside the target under the ordinary umask,
                // outlasting the declaration that made it.
                if let Ok(existing) = self.core.executor.read_file(&target_path).await {
                    if existing == plaintext {
                        apply_declared_owner(&target_path, spec, &self.core.executor).await?;
                        debug!("Link: {:?} is already up-to-date.", target_path);
                        continue;
                    }
                }
                // T5: restricted before it lands, not chmod'd after.
                self.core
                    .executor
                    .write_secret(&target_path, &plaintext)
                    .await?;
                apply_declared_owner(&target_path, spec, &self.core.executor).await?;
                info!("Link: Writing managed secret {:?}", target_path);
                continue;
            }

            // Mode B: Rendered template read from a source file.
            if spec.options.one("template") == Some("true") {
                let rendered = self
                    .core
                    .render_template(&source, spec.options.one("user"))
                    .await?;
                self.core
                    .apply_managed_content(&target_path, &rendered, backup)
                    .await?;
                apply_declared_owner(&target_path, spec, &self.core.executor).await?;
                continue;
            }

            // Mode C: Standard Symlinking Path
            let exists = tokio::fs::try_exists(&target_path).await.unwrap_or(false);
            let is_symlink = target_path.is_symlink();

            if exists || is_symlink {
                if let Ok(existing_link) = tokio::fs::read_link(&target_path).await {
                    if existing_link == source {
                        apply_declared_owner(&target_path, spec, &self.core.executor).await?;
                        debug!("Link: Correct symlink already exists at {:?}", target_path);
                        continue;
                    }
                }

                // A copy Shall made is as much "already in effect" as a symlink it made.
                // Windows without the symlink privilege gets a copy, and asking only
                // `read_link` meant every later sync backed up its own copy as
                // `<target>.shall-backup` and wrote the file again, under a summary
                // reading `already up to date`.
                if exists && !is_symlink {
                    if let (Ok(from), Ok(to)) = (
                        tokio::fs::read(&source).await,
                        tokio::fs::read(&target_path).await,
                    ) {
                        if from == to {
                            apply_declared_owner(&target_path, spec, &self.core.executor).await?;
                            debug!("Link: {:?} already matches {:?}", target_path, source);
                            continue;
                        }
                    }
                }

                // Preserve a pre-existing real file before replacing it with our symlink.
                if backup {
                    self.core.backup_once(&target_path).await?;
                }

                if self.core.executor.dry_run {
                    let kind = if is_symlink {
                        "link"
                    } else if target_path.is_dir() {
                        "directory"
                    } else {
                        "file"
                    };
                    crate::would!("Would replace existing {} at {:?}", kind, target_path);
                } else {
                    crate::utils::file::remove_deployed_path(&target_path)
                        .await
                        .map_err(Error::Io)?;
                }
            }

            info!("Link: Creating link {:?} -> {:?}", source, target_path);

            // Delegate to the executor so the dry-run VFS records this instead of the
            // real filesystem being touched.
            // A symlink is attempted for every target, including one on another drive: a
            // Windows symlink stores the destination as a string and resolves it on open, so
            // it spans volumes. Only the privilege varies, so only the privilege is handled.
            #[cfg(target_os = "windows")]
            {
                match self.core.executor.symlink(&source, &target_path).await {
                    Err(e) if is_missing_symlink_privilege(&e) => {
                        warn!("{}", copy_fallback_message(&source, &target_path));
                        tokio::fs::copy(&source, &target_path)
                            .await
                            .map_err(Error::from)?;
                    }
                    other => other?,
                }
            }

            #[cfg(unix)]
            {
                self.core.executor.symlink(&source, &target_path).await?;
            }
            apply_declared_owner(&target_path, spec, &self.core.executor).await?;
        }
        Ok(())
    }

    /// Undo a `link:` declaration. Each name is the DESTINATION Shall wrote — never the source
    /// in your repo, which Shall does not own and must not delete.
    ///
    /// **A declaration undoes what it did (T6).** If a `<target>.shall-backup` is sitting there,
    /// the target was somebody's file before Shall took it over: the backup is put back and the
    /// backup file removed, so a `link:` line that comes and goes leaves the machine as it found
    /// it and nothing accumulates. With no backup there was nothing there before, so the target
    /// is removed.
    async fn remove(
        &self,
        names: &[String],
        _: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        for name in names {
            let path = Path::new(name);
            let exists = tokio::fs::try_exists(path).await.unwrap_or(false);
            let is_symlink = path.is_symlink();
            let backup = backup_path(path);
            let has_backup = tokio::fs::try_exists(&backup).await.unwrap_or(false);

            if !exists && !is_symlink && !has_backup {
                continue;
            }

            // Ownership is not proven by the absence of a backup. `link:` places links and
            // files only, so a real directory sitting at the target with no `.shall-backup`
            // behind it cannot be something Shall put there — it is the user's own tree, and
            // deleting it on remove is how uninstalling a dotfile cost someone their config.
            // Checked before the dry-run preview, so a preview predicts the refusal instead
            // of promising a removal that will never be allowed.
            if (exists && !is_symlink && !has_backup)
                && tokio::fs::symlink_metadata(path)
                    .await
                    .map(|m| m.is_dir())
                    .unwrap_or(false)
            {
                return Err(Error::Other(format!(
                    "`{}` is a directory, and `link:` places links and files — Shall did not \
                     put it there and has no backup of anything it replaced there, so it is \
                     not removing it. Move it aside or delete it yourself if it is yours.",
                    path.display()
                )));
            }

            if self.core.executor.dry_run {
                if has_backup {
                    crate::would!("Link: would restore {:?} from {:?}", path, backup);
                } else {
                    crate::would!("Link: would remove {:?}", path);
                }
                continue;
            }

            if exists || is_symlink {
                crate::utils::file::remove_deployed_path(path)
                    .await
                    .map_err(Error::Io)?;
            }

            if has_backup {
                // A failed restore leaves the user with neither their file nor an error, so
                // it propagates rather than being logged past.
                tokio::fs::rename(&backup, path)
                    .await
                    .map_err(Error::from)?;
                info!(
                    "Link: {:?} restored from the backup taken when it was declared.",
                    path
                );
            } else {
                info!("Link: removed {:?}", path);
            }
        }
        Ok(())
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    cfg: &crate::config::Config,
) {
    // Declared secret providers (U38), through the same approved loader every adapter file uses.
    let layout = cfg.layout();
    let secret_providers = crate::backends::onboarder::read_approved_definitions(
        &layout.adapter_secret_file(),
        &layout.locks_dir(),
    )
    .and_then(
        |body| match toml::from_str::<crate::model::secret::SecretProviderFile>(&body) {
            Ok(f) => Some(crate::model::secret::providers(f.secret)),
            Err(e) => {
                tracing::warn!(
                    "{}",
                    crate::app::adapters::cannot_use(
                        crate::app::adapters::surface("secret").expect("a declared surface"),
                        e,
                    )
                );
                None
            }
        },
    )
    .unwrap_or_default();

    let core = Arc::new(
        LinkBackendCore::new(exec.clone(), Arc::new(cfg.clone()))
            .with_secret_providers(secret_providers),
    );
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(core.clone())
            .with_installable(Arc::new(LinkInstallable { core: core.clone() }))
            .with_metadata_provider(core.clone())
            .build(),
    ));
}
