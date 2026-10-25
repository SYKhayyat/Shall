use crate::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result, Searchable, Upgradable,
};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

/// The name every brew verb takes the exclusive lock under.
///
/// Asked of `stale_lock`, which owns the table of which programs share one package
/// database, rather than spelled as a literal here — a second copy of that table is
/// exactly what its own doc says goes stale. A verb that changes the manager takes
/// the manager's lock; install and remove already did, and `update` and the cache
/// cleaners did not.
fn lock_key() -> &'static str {
    crate::app::stale_lock::lock_key("brew")
}

pub struct BrewBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
}

impl BrewBackendCore {
    pub fn new(executor: CommandExecutor) -> Self {
        Self {
            executor: executor.with_exit_policy(crate::core::exit_policy::for_manager("brew")),
            name: "brew".to_string(),
        }
    }
}

#[async_trait]
impl BackendCore for BrewBackendCore {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        self.executor.command_exists_sync("brew")
    }
    fn probes(&self) -> Vec<String> {
        vec!["brew".into()]
    }
    fn needs_root(&self) -> bool {
        false
    }
}

#[async_trait]
impl MetadataProvider for BrewBackendCore {
    async fn get_dependencies(&self, name: &str) -> Result<Vec<String>> {
        let mut args = vec!["deps".to_string()];
        crate::core::argv::push_names(&mut args, "brew", [name]);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = self.executor.run_output("brew", &arg_refs, false).await?;
        Ok(output
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect())
    }
}

pub struct BrewInstallable {
    pub core: Arc<BrewBackendCore>,
}

#[async_trait]
impl Installable for BrewInstallable {
    /// One `brew install` for every formula, not one each (`Q45`).
    ///
    /// brew resolves the dependency graph per invocation, and this runs under
    /// `run_exclusive` — so N packages one at a time is N resolutions *and* N serialised
    /// lock acquisitions, for a command that takes a list.
    async fn install(&self, specs: &[PackageSpec], _sudo: bool) -> Result<()> {
        if specs.is_empty() {
            return Ok(());
        }
        // **No version goes on this command line** (`Q53`, `S85`). `name@version` is a different
        // formula's *name* in Homebrew — `python@3.12`, `openssl@3` — not a version selector, and
        // building one from a full version names a formula that does not exist. `brew install
        // tokei@14.0.0` answers *No available formula*, and because `lock` records the version it
        // observes, the sync that fed it back failed on a pin the user never typed and failed
        // that way for ever. A pin that cannot be honoured is refused by the planner, by name,
        // before anything runs; it is never built and hoped for here.
        let targets: Vec<String> = specs.iter().map(|spec| spec.name.clone()).collect();
        info!("Brew: Installing {} formula(e)...", targets.len());
        let mut args = vec!["install".to_string()];
        crate::core::argv::push_names(&mut args, "brew", targets.iter().map(String::as_str));
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.core
            .executor
            .run_exclusive(lock_key(), "brew", &arg_refs, false)
            .await?;
        Ok(())
    }

    async fn remove(
        &self,
        names: &[String],
        _sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        if names.is_empty() {
            return Ok(());
        }
        {
            info!("Brew: Uninstalling {} formula(e)...", names.len());
            let mut args = vec!["uninstall".to_string()];
            crate::core::argv::push_names(&mut args, "brew", names.iter().map(String::as_str));
            let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
            self.core
                .executor
                .run_exclusive(lock_key(), "brew", &arg_refs, false)
                .await?;
        }
        Ok(())
    }
}

pub struct BrewQueryable {
    pub core: Arc<BrewBackendCore>,
}

#[async_trait]
impl Queryable for BrewQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), &self.core.name)
    }

    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        let output = self
            .core
            .executor
            .run_output("brew", &["list", "--versions"], false)
            .await?;
        Ok(crate::parsers::common::parse_simple_list(&output, "brew")?)
    }

    async fn list_manual(&self) -> Result<Vec<Package>> {
        let output = self
            .core
            .executor
            .run_output("brew", &["leaves"], false)
            .await?;
        Ok(output
            .lines()
            .map(|l| Package::new(l.trim(), "brew"))
            .collect())
    }

    /// Uses `brew info --json=v1`, the only form that reports the install path.
    ///
    /// **`brew info` answers about the formula, not about this machine.** It prints a full
    /// record for anything in a tapped repository, installed or not, with `"installed": []` as
    /// the only thing that distinguishes the two — and that array was read for its properties
    /// while its emptiness was ignored. So `info` said `Some` for every formula Homebrew has
    /// ever heard of: `shall install brew:jq` reported *already up to date* and installed
    /// nothing, and `shall info jq` told the user a package was on their machine because a tap
    /// knows the name.
    ///
    /// **And the version came from `versions.stable`, which is the newest published one.** A
    /// declaration pinning the version a machine actually has compared against upstream's
    /// latest instead, so every `brew:x@version=` re-installed on every sync from the moment
    /// Homebrew published a newer bottle. The installed keg carries its own version; that is
    /// the one a drift check is about.
    async fn info(&self, name: &str) -> Result<Option<Package>> {
        let mut args = vec!["info".to_string(), "--json=v1".to_string()];
        crate::core::argv::push_names(&mut args, "brew", [name]);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = self
            .core
            .executor
            .run_output("brew", &arg_refs, false)
            .await?;
        if output.is_empty() || output == "[]" {
            return Ok(None);
        }
        let json: Value = serde_json::from_str(&output)
            .map_err(|e| Error::Other(format!("Brew JSON error: {}", e)))?;
        let arr = json
            .as_array()
            .ok_or_else(|| Error::Other("Expected array".into()))?;
        if arr.is_empty() {
            return Ok(None);
        }
        let first = &arr[0];
        let Some(installed) = first["installed"].as_array().and_then(|a| a.last()) else {
            return Ok(None);
        };
        let pkg_name = first["name"].as_str().unwrap_or(name).to_string();
        // The keg's own version. `brew` lists kegs oldest-first, so the last is the one a
        // freshly linked formula is running.
        let version = installed["version"]
            .as_str()
            .or_else(|| first["versions"]["stable"].as_str())
            .unwrap_or("unknown");
        let mut pkg = Package::with_version(&pkg_name, version, "brew");
        if let Some(path) = installed["installed_as_dependency"].as_bool() {
            pkg.properties
                .insert("installed_as_dependency".to_string(), path.to_string());
        }
        // The install path is the prefix of the installed keg
        if let Some(prefix) = installed["prefix"].as_str() {
            pkg.properties
                .insert("install_path".to_string(), prefix.to_string());
        }
        // Fallback: use the cellar path
        if !pkg.properties.contains_key("install_path") {
            if let Some(cellar) = first["cellar"].as_str() {
                pkg.properties.insert(
                    "install_path".to_string(),
                    format!("{}/{}", cellar, pkg_name),
                );
            }
        }
        Ok(Some(pkg))
    }
}

pub struct BrewSearchable {
    pub core: Arc<BrewBackendCore>,
}

#[async_trait]
impl Searchable for BrewSearchable {
    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let mut args = vec!["search".to_string()];
        crate::core::argv::push_names(&mut args, "brew", [query]);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = self
            .core
            .executor
            .search_output("brew", &arg_refs, false)
            .await?;
        Ok(parse_brew_search(&output))
    }

    /// `brew outdated --json=v2` — every formula and cask with an update, in one call (`Q44`).
    async fn outdated_all(&self) -> Result<Option<Vec<Package>>> {
        let output = self
            .core
            .executor
            .run_output("brew", &["outdated", "--json=v2"], false)
            .await?;
        Ok(Some(crate::parsers::common::parse_brew_outdated(&output)))
    }
}

/// Parse `brew search <q>` — one formula/cask name per line, with "==> Formulae" /
/// "==> Casks" section headers to skip.
pub fn parse_brew_search(output: &str) -> Vec<Package> {
    let mut results = Vec::new();
    for line in output.lines() {
        let name = line.trim();
        if name.is_empty() || name.starts_with("==>") {
            continue;
        }
        results.push(Package::new(name, "brew"));
    }
    results
}

pub struct BrewUpgradable {
    pub core: Arc<BrewBackendCore>,
}

#[async_trait]
impl Upgradable for BrewUpgradable {
    async fn update(&self, _sudo: bool) -> Result<()> {
        // `brew update` rewrites taps — manager state that `install` locks. A verb that
        // changes the manager takes the manager's lock; install and remove did and this did
        // not, which left the partition `managers.rs` relies on resting on the fact that
        // nothing currently runs two brew verbs at once.
        self.core
            .executor
            .run_exclusive(lock_key(), "brew", &["update"], false)
            .await?;
        Ok(())
    }
    async fn upgrade(&self, _sudo: bool) -> Result<()> {
        self.core
            .executor
            .run_exclusive(lock_key(), "brew", &["upgrade"], false)
            .await?;
        Ok(())
    }
    async fn list_orphans(&self) -> Result<Vec<String>> {
        let out = self
            .core
            .executor
            .run_output("brew", &["autoremove", "--dry-run"], false)
            .await?;
        // `--dry-run` prints "Would remove: a b c" plus prose; the formula names are the
        // lines that are a bare token, which is what every other brew listing looks like.
        Ok(out
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.contains(' ') && !l.starts_with("==>"))
            .map(|l| l.to_string())
            .collect())
    }

    async fn clean_cache(&self, _sudo: bool) -> Result<()> {
        // `brew cleanup` deletes old versions and cached bottles. Same argument as `update`.
        self.core
            .executor
            .run_exclusive(lock_key(), "brew", &["cleanup"], false)
            .await?;
        Ok(())
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    _cfg: &crate::config::Config,
) {
    let core = Arc::new(BrewBackendCore::new(exec.clone()));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(core.clone())
            .with_installable(Arc::new(BrewInstallable { core: core.clone() }))
            .with_queryable(Arc::new(BrewQueryable { core: core.clone() }))
            .with_searchable(Arc::new(BrewSearchable { core: core.clone() }))
            .with_upgradable(Arc::new(BrewUpgradable { core: core.clone() }))
            .with_metadata_provider(core.clone())
            .build(),
    ));
}
