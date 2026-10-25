// src/backends/registry.rs

use crate::app::LuaHooks;
use crate::backends::generic::RepoListing;
use crate::backends::generic::{
    GenericBackendCore, GenericInstallable, GenericQueryable, GenericSearchable, GenericUpgradable,
    ManagerConfig, ManualListing, SearchSource,
};
use crate::config::Config;
use crate::core::{BackendCapabilities, CommandExecutor};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::trace;

/// The registrations, grouped by what the managers have in common rather than by how many
/// lines each takes. This file was 4,237 lines of which 1,800 were `fn register_*` bodies in
/// declaration order, so "where does apt live" and "what else is like apt" had the same answer:
/// scroll.
/// **Public for the integration tests' sake.** The registry mirror links the library without
/// `cfg(test)` and drives the module-owned registrars by name.
#[doc(hidden)]
pub mod language;
#[doc(hidden)]
pub mod os_native;
#[doc(hidden)]
pub mod system;

use language::*;
use os_native::*;
use system::*;

/// **Ordered, because everything downstream walks it and calls the result an order.**
///
/// This was a `HashMap`, whose iteration order Rust randomises per process — so `available()`
/// and `all()` handed back the backends in a different sequence on every run. Two `shall list`
/// runs a second apart differed by 530 lines and sorted to the same file; `check health` moved
/// its rows; the fan-outs handed their first slots to whichever managers the seed picked, so no
/// timing measurement was reproducible; and any code that takes the *first* backend that can
/// answer was tossing a coin. A map keyed by a name people read is a map that should come out
/// in an order people can predict.
pub struct BackendRegistry {
    backends: BTreeMap<String, Arc<BackendCapabilities>>,
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BackendRegistry {
    pub fn new() -> Self {
        Self {
            backends: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, backend: Arc<BackendCapabilities>) {
        let name = backend.name().to_string();
        trace!("Registry: Cataloging backend '{}'", name);
        self.backends.insert(name, backend);
    }

    pub fn get(&self, name: &str) -> Option<Arc<BackendCapabilities>> {
        self.backends.get(name).cloned()
    }

    /// Whether this host can actually run anything through that manager (II.7c).
    ///
    /// **The two ways to answer no are one answer to the user.** A name this build never
    /// registered — `apt` on Windows, `winget` on Linux — and a name registered here whose
    /// program is not installed are different facts about the registry and the same fact about
    /// the machine: there is nothing to install through, and nothing installed to remove.
    /// Written once so no call site can handle one and forget the other, which is how
    /// `spec_is_missing` came to raise `BackendNotFound` for the first and plan an install that
    /// could not run for the second.
    ///
    /// This does **not** answer "is that a real backend name" — a typo is caught in the
    /// grammar, against `Vocab`, before anything gets here.
    pub fn runs_here(&self, name: &str) -> bool {
        self.get(name).is_some_and(|b| b.is_available())
    }

    /// Whether an exact `@version=` can be turned into an install argument here (`Q53`).
    ///
    /// **A backend that cannot install at all cannot replay a version either**, so a name with
    /// no `Installable` answers no — the same shape as `runs_here`, where the two ways to answer
    /// no are one answer to the user.
    pub fn pins_version(&self, name: &str) -> bool {
        self.get(name)
            .and_then(|b| b.as_installable().map(|i| i.pins_version()))
            .unwrap_or(false)
    }

    /// Every registered backend whose program is installed here.
    ///
    /// **Named for the question it answers, because it used to be called `available()` and was
    /// used for a different one.** "Which backends are on this machine" and "which backends may
    /// Shall use" are not the same question — `priority` decides the second — and one method
    /// answering both is how the priority file came to gate resolution and nothing else. The
    /// second question lives on [`crate::app::Backends`], which filters by name *before*
    /// probing; this one probes everything by construction, so a caller reaching for it is
    /// asserting that the machine, and not the config, is its subject.
    pub fn present_on_this_machine(&self) -> Vec<Arc<BackendCapabilities>> {
        self.backends
            .values()
            .filter(|b| b.is_available())
            .cloned()
            .collect()
    }

    /// Every backend this build knows about, installed or not.
    ///
    /// `check health`'s subject: a manager that is absent is a report, and a manager that is
    /// absent *and named by `priority`* is a failure — a distinction only visible from here.
    pub fn all(&self) -> Vec<Arc<BackendCapabilities>> {
        self.backends.values().cloned().collect()
    }
}

/// Build the default backend registry.
///
/// This is a thin orchestrator: each specialized backend owns its own
/// `register(reg, exec, cfg)` in its module, and the generic (CLI-config-driven)
/// backends are registered by the small `register_*` helpers below. Adding a backend
/// is a localized change — write its `register` and add one call here.
pub async fn create_default_registry(
    executor: CommandExecutor,
    config: &Config,
    _hooks: Arc<LuaHooks>,
) -> BackendRegistry {
    let mut reg = BackendRegistry::new();

    // --- Linux native system managers ---
    if cfg!(target_os = "linux") {
        register_apt(&mut reg, &executor);
        register_apk(&mut reg, &executor);
        register_zypper(&mut reg, &executor);
        register_pacman(&mut reg, &executor);
        register_dnf(&mut reg, &executor);
        register_xbps(&mut reg, &executor);
        // AUR helpers: pacman-syntax drop-ins for Arch's user repository. Registered as
        // distinct backends (not a pacman flag) so `yay:pkg` / `paru:pkg` are explicit and
        // tracked separately. Runtime-gated by the helper binary being present.
        register_yay(&mut reg, &executor);
        register_paru(&mut reg, &executor);
    }

    // --- Windows native system managers ---
    if cfg!(target_os = "windows") {
        register_winget(&mut reg, &executor);
        register_scoop(&mut reg, &executor);
        register_choco(&mut reg, &executor);
    }

    // --- macOS native system managers ---
    if cfg!(target_os = "macos") {
        register_mas(&mut reg, &executor);
        register_macports(&mut reg, &executor);
    }

    // --- Everything that is a row in `builtin_backends.toml` ---
    //
    // Registered FIRST so a hand-written registration below is the one that survives a
    // collision, and `no_backend_is_both_a_row_and_a_registrar` in the suite fails when there
    // is one — a name in both places would otherwise be decided by the order of two calls.
    crate::backends::onboarder::register_builtin_backends(&mut reg, &executor);

    // --- Cross-platform & specialized backends (each module owns its registration) ---
    crate::backends::brew::register(&mut reg, &executor, config);
    crate::backends::mise::register(&mut reg, &executor, config);
    crate::backends::github::register(&mut reg, &executor, config);
    crate::backends::web::register(&mut reg, &executor, config);
    crate::backends::btrfs::register(&mut reg, &executor, config);
    crate::backends::storage::register(&mut reg, &executor, config);
    crate::backends::link::register(&mut reg, &executor, config);
    crate::backends::dir::register(&mut reg, &executor, config);
    crate::backends::nix::register(&mut reg, &executor, config);
    crate::backends::nixos::register(&mut reg, &executor, config);
    crate::backends::vscode::register(&mut reg, &executor, config);
    crate::backends::emacs::register(&mut reg, &executor, config);
    crate::backends::service::register(&mut reg, &executor, config);
    crate::backends::setting::register(&mut reg, &executor, config);
    crate::backends::appimage::register(&mut reg, &executor, config);
    crate::backends::snap::register(&mut reg, &executor, config);
    crate::backends::flatpak::register(&mut reg, &executor, config);
    register_conda(&mut reg, &executor, config);
    if cfg!(target_os = "windows") {
        crate::backends::psresource::register(&mut reg, &executor, config);
    }

    // --- Language package managers (generic, config-driven) ---
    register_pip(&mut reg, &executor);
    register_gem(&mut reg, &executor);
    register_bun(&mut reg, &executor);
    // **OS-gated at registration, not by probe.** These three were registered on every OS
    // behind mere binary existence, so an unrelated program called `pkg` (an npm-global
    // `vercel pkg`, say) made FreeBSD's backend report READY here and drove the wrong program.
    // A manager bound to one packaging universe is compiled in only where that universe lives.
    #[cfg(target_os = "netbsd")]
    register_pkgin(&mut reg, &executor);
    #[cfg(target_os = "freebsd")]
    register_pkg_freebsd(&mut reg, &executor);
    #[cfg(target_os = "openbsd")]
    register_pkg_add_openbsd(&mut reg, &executor);
    register_dotnet(&mut reg, &executor);

    // --- Ecosystem backends implemented as dedicated modules (subcommand binary / fs) ---
    crate::backends::go::register(&mut reg, &executor, config);

    // --- User-defined backends (the onboarder). Loaded last so a custom definition
    // can never silently shadow a built-in; collisions are skipped with a warning. ---
    crate::backends::onboarder::load_default_custom_backends(&mut reg, &executor, config);

    reg
}

// ============================================================================
// Ecosystem backends (added in the backend-expansion work)
//
// These all fit the generic CLI-config model. To cut the 20-field `ManagerConfig`
// boilerplate, `base_config` fills in inert defaults and each `register_*` overrides only
// the fields it needs; `register_generic` attaches the requested capability set.
// ============================================================================

/// A `ManagerConfig` with everything defaulted to "off"; callers set the fields they use.
///
/// `pub(crate)` because a test fixture that spells out all thirty-odd fields by hand is a
/// second copy of this list, and the copy stops matching the day a field is added.
/// **`pub` for the integration tests' sake.** The registry mirror constructs one generic core
/// to assert policy propagation without copying the default configuration.
pub fn base_config(name: &str) -> ManagerConfig {
    ManagerConfig {
        name: name.into(),
        binary: None,
        remove_binary: None,
        install_args: vec![],
        remove_args: vec![],
        purge_args: None,
        list_args: vec![],
        // Default to the safe answer, not the convenient one: an unlabelled backend is one
        // nobody has confirmed can separate user-chosen packages from dependencies, so
        // `adopt` adopts nothing from it. A backend whose installed set really is all
        // user-chosen says so with `ManualListing::AllInstalled`.
        manual: ManualListing::Unsupported,
        essential_args: None,
        search_args: vec![],
        search_binary: None,
        enumerate_args: None,
        enumerate_binary: None,
        list_binary: None,
        upgrade_args: vec![],
        update_args: None,
        orphan_dry_run: None,
        foreign_args: None,
        repo_add_args: None,
        repo_remove_args: None,
        repo_list_args: None,
        repo_binary: None,
        repo_list_binary: None,
        repo_remove_binary: None,
        repo_list_shape: RepoListing::Columns,
        depends: None,
        clean_cache: None,
        version_pin: None,
        needs_root: false,
        is_exclusive: false,
        install_source_option: None,
        extra_probes: None,
        upgrade_reinstall_args: None,
        property_probes: Vec::new(),
        machine_list: None,
        outdated: None,
        search_source: SearchSource::Command,
        qualified_names: false,
    }
}

/// Give a generic core its manager's exit policy.
///
/// Named rather than inlined so it can be asserted: an exit policy changes no argv, so a
/// backend that lost one looks identical from everywhere except a failing install.
/// **`pub` for the integration tests' sake.** The registry mirror asserts the manager policy on
/// the same wrapper used by registration.
pub fn with_manager_policy(core: Arc<GenericBackendCore>) -> Arc<GenericBackendCore> {
    Arc::new(GenericBackendCore {
        name: core.name.clone(),
        executor: core
            .executor
            .clone()
            .with_exit_policy(crate::core::exit_policy::for_manager(&core.name)),
        config: core.config.clone(),
        parser: core.parser.clone(),
    })
}

/// Register a generic backend, attaching Installable + MetadataProvider always and the
/// other capabilities per the boolean flags. Installable is always present (install is the
/// point); `query`/`search`/`upgrade` are opt-in because not every manager supports them.
///
/// **Every generic backend gets its manager's exit policy here**, so no registrar can forget
/// it. Two did: converting `cargo` and `pipx` to data on 2026-08-04 dropped the
/// `with_exit_policy` line their hand-written modules had, and `cargo install
/// <no-such-crate>` stopped being classified `permanent` — which sends the sweep harness back
/// to retrying a crate that will never exist. The argv table could not catch it, because an
/// exit policy is not argv. An unknown manager yields the default policy, which classifies
/// nothing, so applying this to all of them is safe in the direction that keeps a declaration.
#[allow(clippy::fn_params_excessive_bools)]
fn register_generic(
    reg: &mut BackendRegistry,
    core: Arc<GenericBackendCore>,
    query: bool,
    search: bool,
    upgrade: bool,
) {
    let core = with_manager_policy(core);
    let mut builder = BackendCapabilities::builder(core.clone())
        .with_installable(Arc::new(GenericInstallable { core: core.clone() }))
        .with_metadata_provider(core.clone());
    if query {
        builder = builder.with_queryable(Arc::new(GenericQueryable { core: core.clone() }));
    }
    if search {
        builder = builder.with_searchable(Arc::new(GenericSearchable { core: core.clone() }));
    }
    if upgrade {
        builder = builder.with_upgradable(Arc::new(GenericUpgradable { core: core.clone() }));
    }
    reg.register(Arc::new(builder.build()));
}
