use crate::core::{
    BackendCore, CommandExecutor, Installable, MetadataProvider, Package, PackageSpec, Queryable,
    Result, Searchable, Upgradable,
};
use crate::model::scope::Scope;
use crate::parsers::{or_unrecognised, ParseResult};
use crate::utils::text::sanitize;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// The name every flatpak verb takes the exclusive lock under.
///
/// Asked of `stale_lock`, which owns the table of which programs share one package
/// database, rather than spelled as a literal here — a second copy of that table is
/// exactly what its own doc says goes stale. A verb that changes the manager takes
/// the manager's lock; install and remove already did, and `update` and the cache
/// cleaners did not.
fn lock_key() -> &'static str {
    crate::app::stale_lock::lock_key("flatpak")
}

pub struct FlatpakBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
    /// Whose installation flatpak acts on. Parsed once at registration: two call sites reading
    /// the raw map is two chances to disagree about what the string meant.
    pub scope: Scope,
}

impl FlatpakBackendCore {
    pub fn new(executor: CommandExecutor, scope: Scope) -> Self {
        Self {
            executor,
            name: "flatpak".to_string(),
            scope,
        }
    }

    /// **These go AFTER the subcommand, and every caller here does that.**
    ///
    /// `flatpak(1)` is `flatpak [OPTION…] COMMAND`, and its top-level options are `--help`,
    /// `--version`, `--verbose` and friends. `--user`/`--system` belong to the *commands* —
    /// `flatpak-install(1)` is `flatpak install [OPTION…] REF…` — so `flatpak --system install`
    /// is rejected before flatpak decides what it was being asked to do.
    ///
    /// Every argv this backend built had the flag in front for as long as the backend has
    /// existed, and nothing noticed, because flatpak is one of the managers no harness has ever
    /// driven: it needs a session bus, so the container matrix names it as argv-tested-only. An
    /// argv test proves a command line was constructed, not that the tool accepts it. What found
    /// it was `argv_drift_tests`, which asks the real flatpak on the tools image whether it
    /// documents the flags Shall passes, and got back
    ///
    /// ```text
    /// `flatpak  --system` — the tool says: error: unknown option --system
    /// ```
    ///
    /// — with the subcommand slot empty, which is exactly where the flag was going.
    pub fn scope_args(&self) -> Vec<&str> {
        match self.scope {
            Scope::User => vec!["--user"],
            Scope::System => vec!["--system"],
        }
    }

    /// Every installed application, with the branch it is on.
    ///
    /// One reader, because the install path needs the same answer the query path does: it has to
    /// know which branch an app is on *before* it adds another one, and a second listing here
    /// would be a second chance to disagree with the one the planner read.
    async fn installed_refs(&self) -> Result<Vec<Package>> {
        let out = self
            .executor
            .run_output(
                "flatpak",
                &["list", "--app", "--columns=application,version,branch"],
                false,
            )
            .await?;
        Ok(parse_flatpak_list(&out)?)
    }

    /// The `[backend_settings.flatpak]` block as a scope, or the message that says what to
    /// write instead.
    ///
    /// `--user` and `--system` are a value, not a flag: the argv needs the word itself, which
    /// is why the key is `scope` and not the boolean it used to be.
    pub fn scope_from_settings(settings: Option<&HashMap<String, String>>) -> Result<Scope> {
        let Some(settings) = settings else {
            return Ok(FLATPAK_DEFAULT_SCOPE);
        };
        if settings.contains_key("user") {
            return Err(crate::core::Error::Config(format!(
                "`[backend_settings.flatpak]` sets `user`, which flatpak no longer reads. Write \
                 `scope = \"user\"` or `scope = \"system\"` instead — the flag needs the word, \
                 not a boolean. Default is `{FLATPAK_DEFAULT_SCOPE}`."
            )));
        }
        let Some(written) = settings.get("scope") else {
            return Ok(FLATPAK_DEFAULT_SCOPE);
        };
        Scope::parse(written).ok_or_else(|| {
            crate::core::Error::Config(format!(
                "`[backend_settings.flatpak]` has `scope = \"{written}\"`. Scope is {}. \
                 Omitting it means `{FLATPAK_DEFAULT_SCOPE}`.",
                Scope::vocabulary()
            ))
        })
    }
}

/// What `flatpak` itself does when neither flag is passed.
pub const FLATPAK_DEFAULT_SCOPE: Scope = Scope::System;

/// `flatpak list --columns=application,version,branch` — TAB-separated, like `flatpak search`.
///
/// **Tabs, not whitespace.** Most flathub apps carry no version, and flatpak keeps an empty
/// *middle* column as an empty field (`org.gimp.GIMP\t\tstable`) while dropping trailing ones.
/// Split on whitespace and the branch of every versionless app is read as its version — measured
/// against flathub, `--columns=application,version,branch` beside `application,branch,version`.
///
/// **A second row for an application erases its branch.** flatpak installs branches side by side
/// and the listing has no column saying which one is current — `--columns=help` offers none, and
/// the binary carries no such word. An app on two branches is therefore a branch Shall cannot
/// read, and D13's rule is that an unreadable value is left alone: guessing one of the two would
/// schedule a switch on every sync for ever.
pub fn parse_flatpak_list(output: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let mut order: Vec<String> = Vec::new();
    let mut apps: HashMap<String, Package> = HashMap::new();
    for line in &candidates {
        let cols: Vec<&str> = line.split('\t').map(str::trim).collect();
        let Some(name) = cols.first().copied().filter(|s| !s.is_empty()) else {
            continue;
        };
        if let Some(seen) = apps.get_mut(name) {
            seen.properties.remove("channel");
            continue;
        }
        let mut p = Package::new(name, "flatpak");
        p.version = cols
            .get(1)
            .filter(|s| !s.is_empty())
            .map(|s| (*s).to_string());
        if let Some(branch) = cols.get(2).filter(|s| !s.is_empty()) {
            p.properties
                .insert("channel".to_string(), (*branch).to_string());
        }
        order.push(name.to_string());
        apps.insert(name.to_string(), p);
    }
    let found = order.into_iter().filter_map(|n| apps.remove(&n)).collect();
    or_unrecognised("flatpak", found, &candidates)
}

#[async_trait]
impl BackendCore for FlatpakBackendCore {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        // No per-backend cache: the executor memoises every PATH lookup now, which dedupes
        // across the backends that probe the same program too. One backend having its own
        // `OnceCell` while the other forty-four re-probed is exactly the "two of everything"
        // this repo removes.
        self.executor.command_exists_sync("flatpak")
    }
    fn probes(&self) -> Vec<String> {
        vec!["flatpak".into()]
    }

    fn needs_root(&self) -> bool {
        // A user-scoped install writes under `$HOME` and must not be run through sudo.
        self.scope == Scope::System
    }
}

#[async_trait]
impl MetadataProvider for FlatpakBackendCore {
    async fn get_dependencies(&self, name: &str) -> Result<Vec<String>> {
        let mut final_args: Vec<String> = vec!["info".to_string()];
        final_args.extend(self.scope_args().into_iter().map(str::to_string));
        final_args.push("--show-metadata".to_string());
        crate::core::argv::push_names(&mut final_args, "flatpak", [name]);
        let arg_refs: Vec<&str> = final_args.iter().map(String::as_str).collect();

        // Flatpak metadata contains a [Extension] or [Runtime] section.
        // We look for 'runtime=' which is the primary transitive dependency.
        let output = self
            .executor
            .run_output("flatpak", &arg_refs, false)
            .await?;
        let mut deps = Vec::new();

        for line in output.lines() {
            if let Some(runtime) = line.strip_prefix("runtime=") {
                deps.push(runtime.trim().to_string());
            }
        }

        Ok(deps)
    }
}

pub struct FlatpakInstallable {
    pub core: Arc<FlatpakBackendCore>,
}

impl FlatpakInstallable {
    /// Which branch each `@channel` app is on right now, read before the install so the switch
    /// after it can tell *added a second branch* from *installed the first*. Empty — and the
    /// listing unread — when no line asks for a channel.
    async fn branches_now(&self, specs: &[PackageSpec]) -> Result<HashMap<String, String>> {
        if !specs.iter().any(|s| s.options.one("channel").is_some()) {
            return Ok(HashMap::new());
        }
        Ok(self
            .core
            .installed_refs()
            .await?
            .into_iter()
            .filter_map(|p| {
                p.properties
                    .get("channel")
                    .map(|c| (p.name.clone(), c.clone()))
            })
            .collect())
    }
}

/// A flatpak ref is `name/arch/branch`. The arch slot stays empty so flatpak keeps choosing it
/// from the machine; writing `name/branch` would be read as an architecture, not a branch.
pub fn install_ref(spec: &PackageSpec) -> String {
    match spec.options.one("channel") {
        Some(channel) => format!("{}//{}", spec.name, channel),
        None => spec.name.clone(),
    }
}

/// `--or-update`, because a ref that is already there is not an error to Shall.
///
/// `flatpak install` answers `Error: <ref> already installed` and exits non-zero. Every other
/// path here can hand it a ref it already has — a `@channel` whose drift is repaired by the
/// same sync that read it, a package adopted between the plan and the apply — and each one of
/// those used to fail the whole transaction over a machine that was already in the declared
/// state. `--or-update` is flatpak's own answer to that: *"Update install if already
/// installed."*
pub fn install_argv(scope: &[&str], specs: &[PackageSpec]) -> Vec<String> {
    let mut args: Vec<String> = vec!["install".to_string()];
    args.extend(scope.iter().map(|s| s.to_string()));
    args.extend([
        "-y".to_string(),
        "--noninteractive".to_string(),
        "--or-update".to_string(),
    ]);
    let names: Vec<String> = specs.iter().map(install_ref).collect();
    crate::core::argv::push_names(&mut args, "flatpak", names);
    args
}

/// `flatpak make-current <app> <branch>`, the only thing that moves an app from one branch to
/// another.
pub fn make_current_argv(scope: &[&str], name: &str, branch: &str) -> Vec<String> {
    let mut args: Vec<String> = vec!["make-current".to_string()];
    args.extend(scope.iter().map(|s| s.to_string()));
    crate::core::argv::push_names(&mut args, "flatpak", [name, branch]);
    args
}

#[async_trait]
impl Installable for FlatpakInstallable {
    async fn install(&self, specs: &[PackageSpec], sudo: bool) -> Result<()> {
        if specs.is_empty() {
            return Ok(());
        }

        let before = self.branches_now(specs).await?;

        let args = install_argv(&self.core.scope_args(), specs);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

        info!("Flatpak: Installing {} package(s)...", specs.len());
        self.core
            .executor
            .run_exclusive(lock_key(), "flatpak", &arg_refs, sudo)
            .await?;

        // **flatpak has no channel switch.** `snap refresh --channel=` moves a snap; installing
        // `app//beta` next to `app//stable` moves nothing — flatpak keeps both and the launcher
        // still runs the old one. `make-current` is what points it at the declared branch, and
        // it has to happen here: the listing carries no current-branch column, so no later sync
        // can see that this step was skipped.
        for spec in specs {
            let Some(want) = spec.options.one("channel") else {
                continue;
            };
            let Some(had) = before.get(&spec.name) else {
                continue;
            };
            // Compared the way the planner compares it, through the same function. Two answers
            // to "is this the branch the line asked for" is how a switch fires on a machine the
            // planner called converged.
            use crate::backends::capability::channel_risk;
            if channel_risk(had) == channel_risk(want) {
                continue;
            }
            info!(
                "Flatpak: Switching {} from branch {} to {}...",
                spec.name, had, want
            );
            let args = make_current_argv(&self.core.scope_args(), &spec.name, want);
            let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
            self.core
                .executor
                .run_exclusive(lock_key(), "flatpak", &arg_refs, sudo)
                .await?;
        }
        Ok(())
    }

    async fn remove(
        &self,
        names: &[String],
        sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        if names.is_empty() {
            return Ok(());
        }

        let mut args: Vec<String> = vec!["uninstall".to_string()];
        args.extend(self.core.scope_args().into_iter().map(str::to_string));
        args.extend(["-y".to_string(), "--noninteractive".to_string()]);
        crate::core::argv::push_names(&mut args, "flatpak", names);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

        info!("Flatpak: Removing {} package(s)...", names.len());
        self.core
            .executor
            .run_exclusive(lock_key(), "flatpak", &arg_refs, sudo)
            .await?;
        Ok(())
    }
}

pub struct FlatpakQueryable {
    pub core: Arc<FlatpakBackendCore>,
}

#[async_trait]
impl Queryable for FlatpakQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), &self.core.name)
    }

    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        self.core.installed_refs().await
    }

    async fn list_manual(&self) -> Result<Vec<Package>> {
        self.list_installed().await
    }

    async fn info(&self, name: &str) -> Result<Option<Package>> {
        let all = self.installed_listing().await?;
        Ok(all.iter().find(|p| p.name == name).cloned())
    }
}

pub struct FlatpakSearchable {
    pub core: Arc<FlatpakBackendCore>,
}

#[async_trait]
impl Searchable for FlatpakSearchable {
    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let mut args = vec!["search".to_string()];
        crate::core::argv::push_names(&mut args, "flatpak", [query]);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = self
            .core
            .executor
            .search_output("flatpak", &arg_refs, false)
            .await?;
        Ok(parse_flatpak_search(&output))
    }
}

/// Parse `flatpak search <q>` => TAB-separated columns:
/// Name \t Description \t Application ID \t Version \t Branch \t Remotes.
/// The Application ID is the installable identifier, so prefer it as the name.
pub fn parse_flatpak_search(output: &str) -> Vec<Package> {
    let mut results = Vec::new();
    for line in sanitize(output).lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').map(|c| c.trim()).collect();
        let display_name = cols.first().copied().unwrap_or("").trim();
        let app_id = cols
            .get(2)
            .copied()
            .filter(|s| !s.is_empty())
            .unwrap_or(display_name);
        if app_id.is_empty() {
            continue;
        }
        let mut p = Package::new(app_id, "flatpak");
        if let Some(desc) = cols.get(1).filter(|s| !s.is_empty()) {
            p.properties
                .insert("description".to_string(), desc.to_string());
        }
        if let Some(ver) = cols.get(3).filter(|s| !s.is_empty()) {
            p.version = Some(ver.to_string());
        }
        results.push(p);
    }
    results
}

pub struct FlatpakUpgradable {
    pub core: Arc<FlatpakBackendCore>,
}

#[async_trait]
impl Upgradable for FlatpakUpgradable {
    async fn update(&self, sudo: bool) -> Result<()> {
        // Must pass -y --noninteractive (like install/upgrade/clean_orphans), otherwise an
        // automated run blocks on flatpak's interactive confirmation prompt.
        let mut args = vec!["update"];
        args.extend(self.core.scope_args());
        args.extend(["-y", "--noninteractive"]);
        debug!("Flatpak: Refreshing remotes...");
        self.core
            .executor
            .run_exclusive(lock_key(), "flatpak", &args, sudo)
            .await?;
        Ok(())
    }

    async fn upgrade(&self, sudo: bool) -> Result<()> {
        let mut args = vec!["update"];
        args.extend(self.core.scope_args());
        args.extend(["-y", "--noninteractive"]);
        info!("Flatpak: Upgrading all applications...");
        self.core
            .executor
            .run_exclusive(lock_key(), "flatpak", &args, sudo)
            .await?;
        Ok(())
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    cfg: &crate::config::Config,
) {
    // A scope this backend cannot read is not a default to fall back on: `--system` where the
    // user asked for `--user` installs for every account and needs root to do it.
    let scope = match FlatpakBackendCore::scope_from_settings(cfg.backend_settings.get("flatpak")) {
        Ok(scope) => scope,
        Err(e) => {
            tracing::warn!("flatpak: {e}");
            return;
        }
    };
    let core = Arc::new(FlatpakBackendCore::new(exec.clone(), scope));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(core.clone())
            .with_installable(Arc::new(FlatpakInstallable { core: core.clone() }))
            .with_queryable(Arc::new(FlatpakQueryable { core: core.clone() }))
            .with_searchable(Arc::new(FlatpakSearchable { core: core.clone() }))
            .with_upgradable(Arc::new(FlatpakUpgradable { core: core.clone() }))
            .with_metadata_provider(core.clone())
            .build(),
    ));
}
