use crate::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result, Searchable, Upgradable,
};
use crate::utils::text::sanitize;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// The name every snap verb takes the exclusive lock under.
///
/// Asked of `stale_lock`, which owns the table of which programs share one package
/// database, rather than spelled as a literal here — a second copy of that table is
/// exactly what its own doc says goes stale. A verb that changes the manager takes
/// the manager's lock; install and remove already did, and `update` and the cache
/// cleaners did not.
fn lock_key() -> &'static str {
    crate::app::stale_lock::lock_key("snap")
}

pub struct SnapBackendCore {
    pub executor: CommandExecutor,
    pub name: String,
}

impl SnapBackendCore {
    pub fn new(executor: CommandExecutor) -> Self {
        Self {
            executor,
            name: "snap".to_string(),
        }
    }
}

#[async_trait]
impl BackendCore for SnapBackendCore {
    fn name(&self) -> &str {
        "snap"
    }

    fn is_available(&self) -> bool {
        self.executor.command_exists_sync("snap")
    }
    fn probes(&self) -> Vec<String> {
        vec!["snap".into()]
    }

    fn needs_root(&self) -> bool {
        // Snap operations almost always require administrative privileges.
        true
    }
}

#[async_trait]
impl MetadataProvider for SnapBackendCore {
    async fn get_dependencies(&self, name: &str) -> Result<Vec<String>> {
        let mut args = vec!["info".to_string()];
        crate::core::argv::push_names(&mut args, "snap", [name]);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = self.executor.run_output("snap", &arg_refs, false).await?;
        let mut deps = Vec::new();

        // Snap dependencies (base snaps like core22, or content interface connections)
        // are usually identified in the 'notes' or 'connections' but snaps are
        // largely self-contained. We look for 'base:' which is the most common requirement.
        for line in output.lines() {
            if let Some(base) = line.strip_prefix("base:") {
                deps.push(base.trim().to_string());
            }
        }

        Ok(deps)
    }
}

pub struct SnapInstallable {
    pub core: Arc<SnapBackendCore>,
}

pub fn install_args(spec: &PackageSpec) -> Vec<String> {
    let mut args = vec!["install".to_string()];

    if spec.options.one("classic") == Some("true") {
        args.push("--classic".into());
    }

    if let Some(channel) = spec.options.one("channel") {
        args.push("--channel".into());
        args.push(channel.to_string());
    }

    crate::core::argv::push_names(&mut args, "snap", [&spec.name]);
    args
}

/// Snap risk levels, most stable first. Moving toward a *more* stable level is a downgrade —
/// `edge -> stable` gives you an older build (D13).
const RISK_ORDER: &[&str] = &["stable", "candidate", "beta", "edge"];

fn risk_rank(channel: &str) -> Option<usize> {
    let risk = crate::backends::capability::channel_risk(channel);
    RISK_ORDER.iter().position(|r| *r == risk)
}

/// Whether moving from `from` to `to` is a downgrade (a more-stable, less-recent channel). An
/// unknown risk on either side is not called a downgrade — better to under-warn than to cry
/// downgrade on a channel we do not understand.
pub fn is_channel_downgrade(from: &str, to: &str) -> bool {
    match (risk_rank(from), risk_rank(to)) {
        (Some(f), Some(t)) => t < f,
        _ => false,
    }
}

/// What a `snap info` report says about a snap that is installed here — absent when it is not.
///
/// One reading, not three. `is_installed` asked `snap list` and `current_channel` asked
/// `snap info` for facts that are on the same page of the same report, so a snap was interrogated
/// twice per sync and the two answers could disagree across the gap between them.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SnapState {
    pub channel: Option<String>,
    pub classic: bool,
}

/// `snap info`'s account of an installed snap, or `None` when it names one that is not.
///
/// Presence is the `installed:` line, not the exit code: `snap info` answers happily for a snap
/// that only exists in the store, and reading that as installed would send every first install
/// down the refresh path.
pub fn installed_state(output: &str) -> Option<SnapState> {
    let installed = output.lines().find_map(|l| l.strip_prefix("installed:"))?;
    // `installed:  1.85.1  (139)  351MB  classic` — the last field is the notes, and it is the
    // only place `snap info` says whether confinement was relaxed. A snap with no notes ends on
    // its size, which can never spell `classic`.
    let classic = installed
        .split_whitespace()
        .next_back()
        .is_some_and(|notes| notes.split(',').any(|n| n == "classic"));
    Some(SnapState {
        channel: output
            .lines()
            .find_map(|l| l.strip_prefix("tracking:"))
            .map(|v| crate::backends::capability::channel_risk(v).to_string()),
        classic,
    })
}

/// `snap refresh` with whatever the declaration asks to change, or `None` when it asks for
/// nothing (Q20).
///
/// **Both switches ride one refresh.** A snap that needs a channel *and* a confinement change
/// used to get only the channel — the branch that built the refresh looked at `@channel` alone,
/// so writing the two options together silently dropped one.
pub fn refresh_args(name: &str, channel: Option<&str>, to_classic: bool) -> Option<Vec<String>> {
    if channel.is_none() && !to_classic {
        return None;
    }
    let mut a = vec!["refresh".to_string()];
    if let Some(channel) = channel {
        a.push("--channel".to_string());
        a.push(channel.to_string());
    }
    if to_classic {
        a.push("--classic".to_string());
    }
    crate::core::argv::push_names(&mut a, "snap", [name]);
    Some(a)
}

impl SnapInstallable {
    /// What this snap is on the machine right now, read once.
    async fn current_state(&self, name: &str) -> Option<SnapState> {
        let mut args = vec!["info".to_string()];
        crate::core::argv::push_names(&mut args, "snap", [name]);
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = self
            .core
            .executor
            .run_output("snap", &refs, false)
            .await
            .ok()?;
        installed_state(&out)
    }
}

#[async_trait]
impl Installable for SnapInstallable {
    async fn install(&self, specs: &[PackageSpec], sudo: bool) -> Result<()> {
        for spec in specs {
            // D13: `snap install` refuses an already-installed snap, so a `@channel` change has
            // to go through `snap refresh --channel=`. Q20 puts `@classic` on the same footing.
            // Decide by whether the snap is present.
            let want_channel = spec.options.one("channel");
            let want_classic = spec.options.one("classic");

            let args = if let Some(state) = self.current_state(&spec.name).await {
                if let (Some(current), Some(channel)) = (&state.channel, want_channel) {
                    if is_channel_downgrade(current, channel) {
                        // A downgrade is removal-shaped; name it so it is not a silent swap in
                        // the plan the user is confirming.
                        warn!(
                            "Snap: {} is a channel downgrade ({} -> {}) — a less recent build",
                            spec.name, current, channel
                        );
                    }
                }
                // Q20: **snapd relaxes confinement in place and does not narrow it.**
                // `snap refresh --classic` moves a strict snap to classic; there is no switch
                // that moves one back, so the only way is remove-and-reinstall — a removal, of a
                // package the user declared, to satisfy an option. That is the guard's decision
                // and not this backend's, so the line is refused and the way out is named.
                if want_classic == Some("false") && state.classic {
                    return Err(Error::Validation(format!(
                        "`snap:{}` is installed with classic confinement and the line declares \
                         `@classic=false` — snapd can relax confinement in place but cannot \
                         narrow it, so there is no refresh that does this. Remove and reinstall \
                         it by hand (`snap remove {}` then let `sync` install it), or restore \
                         `@classic`.",
                        spec.name, spec.name
                    )));
                }
                let to_classic = want_classic == Some("true") && !state.classic;
                if to_classic {
                    info!("Snap: Moving {} to classic confinement...", spec.name);
                }
                match refresh_args(&spec.name, want_channel, to_classic) {
                    Some(a) => {
                        if let Some(channel) = want_channel {
                            info!("Snap: Switching {} to channel {}...", spec.name, channel);
                        }
                        a
                    }
                    // Present, and neither switch applies: the planner asked for this spec for
                    // some other reason. Falls through to `snap install` exactly as it did
                    // before Q20 — snapd refuses an installed snap and the sync says so, which
                    // is how an unsatisfiable `@version` on a manager that cannot pin one has
                    // always surfaced. Changing that is its own question, not this one's.
                    None => install_args(spec),
                }
            } else {
                info!("Snap: Installing {}...", spec.name);
                install_args(spec)
            };

            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            self.core
                .executor
                .run_exclusive(lock_key(), "snap", &arg_refs, sudo)
                .await?;
        }
        Ok(())
    }

    /// One `snap remove` for every snap (`Q45`).
    ///
    /// **Removal batches; install does not.** `snap install` above has to choose per package
    /// between `install` and `refresh --channel=` depending on what is already present (D13,
    /// Q20), so those specs cannot share a command. Removal asks no such question.
    async fn remove(
        &self,
        names: &[String],
        sudo: bool,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        if names.is_empty() {
            return Ok(());
        }
        info!("Snap: Removing {} snap(s)...", names.len());
        let mut args = vec!["remove".to_string()];
        crate::core::argv::push_names(&mut args, "snap", names.iter().map(String::as_str));
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.core
            .executor
            .run_exclusive(lock_key(), "snap", &arg_refs, sudo)
            .await?;
        Ok(())
    }
}

pub struct SnapQueryable {
    pub core: Arc<SnapBackendCore>,
}

#[async_trait]
impl Queryable for SnapQueryable {
    fn installed_cache(&self) -> (&crate::core::installed::InstalledListings, &str) {
        (self.core.executor.installed_listings(), &self.core.name)
    }

    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        let output = self
            .core
            .executor
            .run_output("snap", &["list"], false)
            .await?;
        let mut packages = Vec::new();

        for line in sanitize(&output).lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let (Some(name), Some(version)) = (parts.first(), parts.get(1)) {
                packages.push(Package::with_version(name, version, "snap"));
            }
        }
        Ok(packages)
    }

    async fn list_manual(&self) -> Result<Vec<Package>> {
        let installed = self.list_installed().await?;
        let base_snaps = [
            "core",
            "core18",
            "core20",
            "core22",
            "snapd",
            "bare",
            "gtk-common-themes",
        ];
        Ok(installed
            .into_iter()
            .filter(|p| !base_snaps.contains(&p.name.as_str()))
            .collect())
    }

    async fn info(&self, name: &str) -> Result<Option<Package>> {
        let mut args = vec!["info".to_string()];
        crate::core::argv::push_names(&mut args, "snap", [name]);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = self
            .core
            .executor
            .run_output("snap", &arg_refs, false)
            .await?;
        if output.is_empty() {
            return Ok(None);
        }

        // **`snap info` answers from the store.** It prints publisher, summary and channels for
        // any snap that exists, installed or not, and the `installed:` line is the only part of
        // that report that is about this machine. Returning `Some` for the whole report told the
        // planner every declared snap was already present, so `shall install snap:x` reported
        // success and installed nothing. `snap find` is the command that answers *does this
        // exist*, and `Searchable` is where it is asked.
        let Some(state) = installed_state(&output) else {
            return Ok(None);
        };
        let mut p = Package::new(name, "snap");
        // Q20: confinement, from the same report and the same line as the version. The planner
        // reads `info`, so a `@classic` that never took effect is invisible without this — the
        // snap is installed, so the name is present, so `sync` reports nothing to do over a
        // declaration it applied to a different confinement.
        p.properties
            .insert("classic".to_string(), state.classic.to_string());
        for line in output.lines() {
            if let Some(v) = line.strip_prefix("summary:") {
                p.properties
                    .insert("summary".to_string(), v.trim().to_string());
            }
            if let Some(v) = line.strip_prefix("installed:") {
                let ver = v.split_whitespace().next().unwrap_or(v);
                p.version = Some(ver.trim().to_string());
            }
            // D13: the channel this snap is following, so a `@channel` change is visible to the
            // planner. `snap info` prints `tracking:     latest/stable`.
            if let Some(v) = line.strip_prefix("tracking:") {
                let risk = crate::backends::capability::channel_risk(v.trim());
                p.properties.insert("channel".to_string(), risk.to_string());
            }
        }
        Ok(Some(p))
    }
}

pub struct SnapSearchable {
    pub core: Arc<SnapBackendCore>,
}

#[async_trait]
impl Searchable for SnapSearchable {
    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let mut args = vec!["find".to_string()];
        crate::core::argv::push_names(&mut args, "snap", [query]);
        let search_args: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = self
            .core
            .executor
            .search_output("snap", &search_args, false)
            .await?;
        Ok(parse_snap_find(&output))
    }
}

/// Parse `snap find <q>` => "Name  Version  Publisher  Notes  Summary" (header + rows).
pub fn parse_snap_find(output: &str) -> Vec<Package> {
    let mut results = Vec::new();
    for line in sanitize(output).lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let Some(name) = parts.first() else { continue };
        let mut p = Package::new(*name, "snap");
        if let Some(version) = parts.get(1) {
            p.version = Some(version.to_string());
        }
        if let Some(publisher) = parts.get(2) {
            p.properties
                .insert("publisher".to_string(), publisher.to_string());
        }
        if parts.len() > 4 {
            p.properties
                .insert("summary".to_string(), parts[4..].join(" "));
        }
        results.push(p);
    }
    results
}

pub struct SnapUpgradable {
    pub core: Arc<SnapBackendCore>,
}

#[async_trait]
impl Upgradable for SnapUpgradable {
    async fn update(&self, sudo: bool) -> Result<()> {
        debug!("Snap: Refreshing all snaps...");
        self.core
            .executor
            .run_exclusive(lock_key(), "snap", &["refresh"], sudo)
            .await?;
        Ok(())
    }

    async fn upgrade(&self, sudo: bool) -> Result<()> {
        self.update(sudo).await
    }
}

pub fn register(
    reg: &mut crate::backends::BackendRegistry,
    exec: &CommandExecutor,
    _cfg: &crate::config::Config,
) {
    let core = Arc::new(SnapBackendCore::new(exec.clone()));
    reg.register(Arc::new(
        crate::core::BackendCapabilities::builder(core.clone())
            .with_installable(Arc::new(SnapInstallable { core: core.clone() }))
            .with_queryable(Arc::new(SnapQueryable { core: core.clone() }))
            .with_searchable(Arc::new(SnapSearchable { core: core.clone() }))
            .with_upgradable(Arc::new(SnapUpgradable { core: core.clone() }))
            .with_metadata_provider(core.clone())
            .build(),
    ));
}
