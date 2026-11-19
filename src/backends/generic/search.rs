//! Search, enumerate, upgrade and repositories — the capabilities that are not about what is
//! already here.

use super::*;

pub struct GenericSearchable {
    pub core: Arc<GenericBackendCore>,
}

#[async_trait]
impl Searchable for GenericSearchable {
    fn qualifies_names(&self) -> bool {
        self.core.config.qualified_names
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        match self.core.config.search_source {
            SearchSource::NpmRegistry => {
                return crate::backends::node_registry::registry_search(query, &self.core.name, 25)
                    .await
            }
            SearchSource::PyPi => {
                return crate::backends::pip_search::registry_search(query, &self.core.name).await
            }
            SearchSource::Command => {}
        }
        let bin = self
            .core
            .config
            .search_binary
            .as_deref()
            .unwrap_or(self.core.binary());
        // A backend reachable as `Searchable` only because it declared an outdated verb has
        // no search command, and running its binary with a bare query would be a different
        // command entirely. "Not configured" is refused by name, never answered as "no
        // results" — the same distinction U2 draws for every other optional capability.
        if self.core.config.search_args.is_empty() {
            return Err(Error::Validation(format!(
                "`{}` was not told how to search, so it cannot answer what it carries. Add \
                 `search_args` to its definition.",
                self.core.name
            )));
        }
        let mut owned: Vec<String> = self.core.config.search_args.clone();
        crate::core::argv::push_names(&mut owned, bin, [query]);
        let args: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        let output = self.core.executor.search_output(bin, &args, false).await?;
        Ok(self.core.parser.parse_search(&output))
    }

    async fn outdated_all(&self) -> Result<Option<Vec<Package>>> {
        let Some(probe) = &self.core.config.outdated else {
            return Ok(None);
        };
        let bin = probe
            .binary
            .as_deref()
            .or(self.core.config.list_binary.as_deref())
            .unwrap_or(self.core.binary());
        let args: Vec<&str> = probe.args.iter().map(|s| s.as_str()).collect();
        // `run_output_maybe_silent`, not `run_output`: several managers report "there are
        // updates" with a non-zero exit — `dnf check-update` returns 100 — and that is an
        // answer, not a fault. The None half is typed at the executor: it is exactly the ran-
        // failed-and-said-nothing case, and never the idle-timeout kill, which errors here and
        // propagates. Matching the old error prose instead is how "no output" from a wedged,
        // killed query once read as zero updates.
        match self
            .core
            .executor
            .run_output_maybe_silent(bin, &args, false)
            .await?
        {
            Some(output) => {
                let parsed = (probe.parse)(&output);
                // **A failed parse must not look like a fact.** The reader type answers in
                // `Vec`s, so an output the reader recognised nothing in comes back
                // indistinguishable from "nothing is outdated" — which is how a localized
                // winget, printing a header no English label matches, silenced every upgrade
                // on the machine for ever. A substantial answer with zero packages read from
                // it is refused, not believed; a real empty answer is one or two lines of
                // header and passes.
                if parsed.is_empty() {
                    let lines = output.lines().filter(|l| !l.trim().is_empty()).count();
                    if lines >= 3 {
                        return Err(Error::command_failed_permanently(format!(
                            "`{bin} {}` printed {lines} lines that its reader recognised \
                             nothing in — that cannot be read as \"nothing is outdated\". The \
                             manager's output may have changed format or language; run it by \
                             hand to see what it prints now.",
                            args.join(" ")
                        )));
                    }
                }
                Ok(Some(parsed))
            }
            None if probe.silence_is_none => {
                // Asked, and the manager's documented way of saying "none". `Some(vec![])` and
                // not `None`: `None` would send the caller round the per-package path for an
                // answer it already has, which is the 771s `Q44` measured.
                Ok(Some(Vec::new()))
            }
            None => Err(Error::command_failed(format!(
                "`{bin} {}` exited without producing an answer",
                args.join(" ")
            ))),
        }
    }
}

pub struct GenericEnumerable {
    pub core: Arc<GenericBackendCore>,
}

#[async_trait]
impl Enumerable for GenericEnumerable {
    async fn available_names(&self) -> Result<Vec<String>> {
        let Some(args) = &self.core.config.enumerate_args else {
            return Err(Error::Other(format!(
                "`{}` cannot list every package it could install.",
                self.core.name
            )));
        };
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let bin = self
            .core
            .config
            .enumerate_binary
            .as_deref()
            .unwrap_or(self.core.binary());
        let output = self.core.executor.run_output(bin, &args, false).await?;
        Ok(output
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    }
}

pub struct GenericUpgradable {
    pub core: Arc<GenericBackendCore>,
}

#[async_trait]
impl Upgradable for GenericUpgradable {
    async fn update(&self, sudo: bool) -> Result<()> {
        if let Some(ref update_args) = self.core.config.update_args {
            let args: Vec<&str> = update_args.iter().map(|s| s.as_str()).collect();
            self.core
                .executor
                .run(self.core.binary(), &args, sudo)
                .await?;
        }
        Ok(())
    }

    async fn upgrade(&self, sudo: bool) -> Result<()> {
        // A manager with no upgrade-all verb upgrades by re-installing what is there. `dart pub
        // global activate <name>` with no version moves that package to latest, and running it
        // over the installed list is the whole of "upgrade everything" for pub. Expressed here
        // rather than in a module, because the shape is the manager's, not the module's.
        if let Some(reinstall_args) = self.core.config.upgrade_reinstall_args.clone() {
            let installed = GenericQueryable {
                core: self.core.clone(),
            }
            .fetch_installed()
            .await?;
            // The install path, with the reinstall verb swapped in — so the terminator rule,
            // the exclusivity rule and every other thing `install` knows apply here too. A
            // second hand-rolled loop is how the six modules this replaces each got their own
            // slightly different one.
            let mut cfg = self.core.config.clone();
            cfg.install_args = reinstall_args;
            let reinstaller = Arc::new(GenericBackendCore {
                name: self.core.name.clone(),
                executor: self.core.executor.clone(),
                config: cfg,
                parser: self.core.parser.clone(),
            });
            let specs: Vec<PackageSpec> = installed
                .into_iter()
                .map(|pkg| PackageSpec {
                    name: pkg.name,
                    backend: self.core.name.clone(),
                    ..Default::default()
                })
                .collect();
            if specs.is_empty() {
                return Ok(());
            }
            let installer = GenericInstallable {
                core: reinstaller.clone(),
            };
            // One command for the lot — `npm install -g a b c` is one resolution and one
            // registry conversation, where forty global packages meant forty of each (`Q46`).
            //
            // **The per-package loop below is not dead code, and removing it would lose the
            // property the old loop existed for**: one package that will not reinstall must
            // not stop the other thirty-nine. So the batch is tried first and the loop is what
            // happens when it fails — the fast path in the ordinary case, the isolating path
            // exactly when something is wrong, which is the only time it was ever worth paying
            // for.
            if installer.install(&specs, sudo).await.is_ok() {
                return Ok(());
            }
            debug!(
                "`{}` could not upgrade {} package(s) in one command; re-trying them \
                 individually so one bad package does not hold up the rest",
                self.core.name,
                specs.len()
            );
            // The loop isolates failures; it does not swallow them. Every retry failing used
            // to fall through to `Ok(())` — exit 0 over zero upgraded packages.
            let total = specs.len();
            let mut failed: Vec<String> = Vec::new();
            for spec in specs {
                if installer
                    .install(std::slice::from_ref(&spec), sudo)
                    .await
                    .is_err()
                {
                    failed.push(spec.name);
                }
            }
            if failed.is_empty() {
                return Ok(());
            }
            return Err(Error::command_failed(format!(
                "`{}` upgraded by reinstalling, and {} of {} package(s) failed: {}",
                self.core.name,
                failed.len(),
                total,
                failed.join(", ")
            )));
        }
        let args: Vec<&str> = self
            .core
            .config
            .upgrade_args
            .iter()
            .map(|s| s.as_str())
            .collect();
        if self.core.config.is_exclusive {
            self.core
                .executor
                .run_exclusive(self.core.lock_key(), self.core.binary(), &args, sudo)
                .await?;
        } else {
            self.core
                .executor
                .run(self.core.binary(), &args, sudo)
                .await?;
        }
        Ok(())
    }

    async fn list_orphans(&self) -> Result<Vec<String>> {
        let Some(dry) = &self.core.config.orphan_dry_run else {
            return Err(crate::core::Error::Unsupported(self.core.name.clone()));
        };
        let args: Vec<&str> = dry.args.iter().map(String::as_str).collect();
        let binary = dry.binary.as_deref().unwrap_or(self.core.binary());
        let out = self.core.executor.run_output(binary, &args, false).await?;
        Ok(out
            .lines()
            .filter_map(|l| l.trim().strip_prefix(&dry.removes_line_prefix))
            .filter_map(|rest| rest.split_whitespace().next())
            .map(|n| n.to_string())
            .collect())
    }

    async fn clean_cache(&self, sudo: bool) -> Result<()> {
        let Some(clean) = &self.core.config.clean_cache else {
            return Err(crate::core::Error::Unsupported("cache cleaning".into()));
        };
        let bin = clean.binary.as_deref().unwrap_or(self.core.binary());
        let args: Vec<&str> = clean.args.iter().map(String::as_str).collect();
        info!("{}: clearing the package cache...", self.core.name);
        // Exclusive on the manager for the same reason an install is: emptying the cache and
        // filling it are the same directory, and `is_exclusive` is the row that says so.
        if self.core.config.is_exclusive {
            self.core
                .executor
                .run_exclusive(self.core.lock_key(), bin, &args, sudo)
                .await?;
        } else {
            self.core.executor.run(bin, &args, sudo).await?;
        }
        Ok(())
    }
}

pub struct GenericRepoManager {
    pub core: Arc<GenericBackendCore>,
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
/// Some backends interpolate repo `{name}`/`{url}` into `sh -c` strings
/// (e.g. apk/apt). Reject shell metacharacters so a crafted argument cannot break out
/// of the intended command.
/// What to add to a manager's own verification refusal, if anything.
///
/// Pure, and enumerated over every case by `verification_advice_tests`, because the wrong
/// sentence here is a message that asks the user for exactly what they already wrote. That is
/// the user-visible half of N-4: `tool_help::accepts_flag` withholds a flag the installed tool
/// does not document, and the advice was `Add @unverified to the line` regardless — so a helm
/// with no way to skip verification answered a refusal by naming a flag it would drop.
pub fn verification_note(
    backend: &str,
    binary: &str,
    flag: &str,
    opting_out: bool,
    withheld: bool,
    message: &str,
) -> Option<String> {
    if opting_out {
        // The flag went out and the manager still refused: its own words are the whole story,
        // and Shall has nothing to add.
        if !withheld {
            return None;
        }
        return Some(format!(
            "the line already says `@unverified`, and this `{}` has no flag that turns its \
             verification off — `{}` is not in its help, so Shall did not send it. There is \
             nothing to add to the line: this version cannot install a source it cannot verify.",
            binary, flag
        ));
    }
    // Only a refusal *about verification* earns either sentence. A network failure carrying no
    // such word is not answered with advice about signatures.
    if !message.to_lowercase().contains("verification") {
        return None;
    }
    if withheld {
        return Some(format!(
            "{} checks a signature before it installs, this source carries none, and this \
             version has no flag that skips the check — `{}` is not in its help. `@unverified` \
             would not help here.",
            backend, flag
        ));
    }
    Some(format!(
        "{} checks a signature before it installs, and this source carries none. Add \
         `@unverified` to the line to accept it as-is.",
        backend
    ))
}

fn reject_shell_meta(field: &str, value: &str) -> Result<()> {
    if value.chars().any(|c| {
        matches!(
            c,
            '\'' | '"' | '`' | '$' | ';' | '&' | '|' | '<' | '>' | '\n' | '\r' | '\\'
                // Spaces and glob characters: apk's removal template embeds `{url}` inside
                // a sed pattern (`sed -i '\|{url}|d' …`), where a space or `*` in the URL
                // mangles the match and either deletes the wrong line or none.
                | ' ' | '*' | '?' | '[' | ']'
        )
    }) {
        return Err(crate::core::Error::Other(format!(
            "Unsafe characters in repo {}: '{}'",
            field, value
        )));
    }
    Ok(())
}

/// A repository name that is about to become part of a FILE PATH.
///
/// `{name}` is an argument a manager parses; `{name_component}` is a path segment Shall builds —
/// `/etc/yum.repos.d/<name>.repo`, `/etc/pacman.d/shall-<name>.conf` — and the difference is
/// that `../../../etc/cron.d/x` is a perfectly ordinary argument and a directory escape. Both
/// hand-written modules validated it and the shared repo path did not, because until dnf and
/// pacman became rows no row put a name in a path.
///
/// Deliberately narrower than [`reject_shell_meta`], which passes `/` and `..`: this is the set
/// a repository is actually named from. apt's PPA identifiers (`ppa:git-core/ppa`) carry a
/// colon and a slash and are *arguments*, not paths — they use `{name}` and are unaffected.
fn validate_path_component(backend: &str, name: &str) -> Result<()> {
    let ok = !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if ok {
        return Ok(());
    }
    Err(crate::core::Error::Validation(format!(
        "`{backend}` writes a repository into a file named after it, so `{name}` has to be a \
         single path segment: letters, digits, `-`, `_` and `.` only. Refusing."
    )))
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
/// The first `{placeholder}` still standing in a template argument, if any.
pub fn find_placeholder(s: &str) -> Option<String> {
    let mut rest = s;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        // An unclosed `{` ends the search: everything after it is one unterminated run, so
        // there is no later `}` that could close a different placeholder.
        let close = after.find('}')?;
        let inner = &after[..close];
        // Any word-shaped content counts, not just lowercase: `{Url}` was a real case-typo
        // that slipped past the lowercase-only filter and reached the manager literally.
        if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Some(format!("{{{}}}", inner));
        }
        rest = &after[close..];
    }
    None
}

/// Refuse an argv that still carries a template placeholder.
///
/// apk's removal row is `sed -i '\|{url}|d' /etc/apk/repositories`. With `{url}` never filled,
/// sed searched for the literal text `{url}`, matched nothing, and **exited 0** — so `run()`
/// saw success and Shall reported a repository removed that was still there. An unfilled
/// placeholder has to be a loud failure, or the next row with a new placeholder repeats that
/// silently.
fn reject_unsubstituted(backend: &str, args: &[String]) -> Result<()> {
    for a in args {
        if let Some(ph) = find_placeholder(a) {
            return Err(crate::core::Error::Other(format!(
                "the `{}` backend's repository command still contains `{}` after substitution, \
                 so it would run against the literal text. Refusing. This is a defect in the \
                 backend definition, not in what you asked for.",
                backend, ph
            )));
        }
    }
    Ok(())
}

impl GenericRepoManager {
    /// The URL of the repository the user named.
    ///
    /// A few managers know a repository only by its URL — `gem sources -r <url>`, apk's line in
    /// `/etc/apk/repositories` — so their removal rows carry `{url}` while the caller has one
    /// identifier. Ask the manager's own listing first; a manager whose listing is the URL
    /// itself (apk prints one field per line, which `list_repos` cannot read as a pair) leaves
    /// the identifier as the only thing that can be it.
    async fn url_for(&self, ident: &str) -> Result<String> {
        // A URL is already the complete operand; asking a failing listing command to confirm it
        // would turn a usable direct removal into an unrelated repository-listing failure.
        if ident.contains("://") || ident.starts_with('/') {
            return Ok(ident.to_string());
        }
        let repos = self.list_repos().await.map_err(|e| {
            crate::core::Error::Other(format!(
                "could not list repositories for `{}` while resolving `{}`: {}",
                self.core.name, ident, e
            ))
        })?;
        if let Some((_, url)) = repos.iter().find(|(n, _)| n == ident) {
            return Ok(url.clone());
        }
        if repos.iter().any(|(_, url)| url == ident) {
            return Ok(ident.to_string());
        }
        Err(crate::core::Error::Other(format!(
            "`{backend}` identifies a repository by its URL, and `{ident}` is neither a URL nor \
             a name `{backend}` reports.\n  \
             Run `shall repo list -b {backend}` and pass the source exactly as it appears there.",
            backend = self.core.name,
            ident = ident
        )))
    }
}

#[async_trait]
impl RepoManager for GenericRepoManager {
    async fn add_repo(&self, name: &str, url: &str, sudo: bool) -> Result<()> {
        reject_shell_meta("name", name)?;
        reject_shell_meta("url", url)?;
        let base_args = self.core.config.repo_add_args.as_ref().ok_or_else(|| {
            crate::core::Error::Other("Repository addition not supported for this backend".into())
        })?;

        let mut final_args = Vec::new();
        let component = if base_args.iter().any(|a| a.contains("{name_component}")) {
            validate_path_component(&self.core.name, name)?;
            Some(name)
        } else {
            None
        };
        for arg in base_args {
            let filled = arg
                .replace("{name_component}", component.unwrap_or_default())
                .replace("{name}", name)
                .replace("{url}", url);
            final_args.push(filled);
        }
        reject_unsubstituted(&self.core.name, &final_args)?;

        let arg_refs: Vec<&str> = final_args.iter().map(|s| s.as_str()).collect();
        info!("Repo: Adding {} to {}...", name, self.core.name);
        self.core
            .executor
            .run(self.core.repo_binary(), &arg_refs, sudo)
            .await?;
        Ok(())
    }

    async fn remove_repo(
        &self,
        name: &str,
        sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        reject_shell_meta("name", name)?;
        let base_args = self.core.config.repo_remove_args.as_ref().ok_or_else(|| {
            crate::core::Error::Other("Repository removal not supported for this backend".into())
        })?;

        // Resolved before anything runs: the URL comes from the manager's own listing or from
        // the identifier, and either way it can land inside an `sh -c` string.
        let url = if base_args.iter().any(|a| a.contains("{url}")) {
            let resolved = self.url_for(name).await?;
            reject_shell_meta("url", &resolved)?;
            Some(resolved)
        } else {
            None
        };

        let component = if base_args.iter().any(|a| a.contains("{name_component}")) {
            validate_path_component(&self.core.name, name)?;
            Some(name)
        } else {
            None
        };
        let final_args: Vec<String> = base_args
            .iter()
            .map(|a| {
                let filled = a
                    .replace("{name_component}", component.unwrap_or_default())
                    .replace("{name}", name);
                match &url {
                    Some(u) => filled.replace("{url}", u),
                    None => filled,
                }
            })
            .collect();
        reject_unsubstituted(&self.core.name, &final_args)?;
        let arg_refs: Vec<&str> = final_args.iter().map(|s| s.as_str()).collect();

        self.core
            .executor
            .run(self.core.repo_remove_binary(), &arg_refs, sudo)
            .await?;
        Ok(())
    }

    async fn list_repos(&self) -> Result<Vec<(String, String)>> {
        let base_args = self.core.config.repo_list_args.as_ref().ok_or_else(|| {
            crate::core::Error::Other("Repository listing not supported for this backend".into())
        })?;
        let arg_refs: Vec<&str> = base_args.iter().map(|s| s.as_str()).collect();
        let output = self
            .core
            .executor
            .run_output(self.core.repo_list_binary(), &arg_refs, false)
            .await?;

        // Bare names, and the source is a separate question per name. Asked once each, and a
        // name whose detail cannot be read keeps its row with an empty source: a repository
        // Shall cannot describe is still a repository the user has, and dropping it here would
        // make `repo remove` unable to find something `repo list` was hiding.
        if let RepoListing::NamesThenDetail(detail) = &self.core.config.repo_list_shape {
            let mut repos = Vec::new();
            for name in output.lines().map(str::trim).filter(|l| !l.is_empty()) {
                let filled: Vec<String> =
                    detail.iter().map(|a| a.replace("{name}", name)).collect();
                let refs: Vec<&str> = filled.iter().map(String::as_str).collect();
                let detail = self
                    .core
                    .executor
                    .run_output(self.core.repo_list_binary(), &refs, false)
                    .await
                    .map_err(|e| {
                        crate::core::Error::Other(format!(
                            "could not read repository `{}` details for `{}`: {}",
                            name, self.core.name, e
                        ))
                    })?;
                let source = detail.lines().next().map(str::trim).unwrap_or_default();
                if source.is_empty() {
                    return Err(crate::core::Error::Unreadable(format!(
                        "repository `{}` returned no source from `{}`",
                        name, self.core.name
                    )));
                }
                repos.push((name.to_string(), source.to_string()));
            }
            return Ok(repos);
        }

        let mut repos = Vec::new();
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Skip dashed separator rows ("--------").
            if trimmed.chars().all(|c| c == '-' || c == '=') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            // Skip obvious table headers (e.g. winget "Name Argument Explicit",
            // scoop "Name Source Updated", dnf "repo id  repo name  status") so they don't show
            // up as repositories.
            let is_header = matches!(
                parts[0],
                "Name" | "NAME" | "Repository" | "Repo" | "Bucket" | "Source" | "repo"
            ) && matches!(
                parts[1],
                "Argument" | "URL" | "Url" | "Source" | "Updated" | "Explicit" | "Enabled" | "id"
            );
            if is_header {
                continue;
            }
            repos.push((parts[0].to_string(), parts[1].to_string()));
        }
        Ok(repos)
    }
}
