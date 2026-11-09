//! Shared test doubles: snapshot providers, task provisioners, and the scaffolding eighteen
//! integration binaries build an `App` out of.
//!
//! **This file is `mock_providers/mod.rs` rather than `mock_providers.rs`, and the directory is
//! the point.** Cargo auto-discovers every `tests/*.rs` as its own test target, so at the top
//! level this became a 716 KB binary containing **zero tests**, linked with `lto = true` and
//! `codegen-units = 1`, and its 312 lines were compiled nineteen times: once as that binary and
//! once inside each `mod mock_providers;` that includes it. A directory is not a target, and
//! `mod mock_providers;` resolves here unchanged — so no caller moved.
#![allow(clippy::field_reassign_with_default, dead_code)]

/// A backend that answers `pins_version` and `supports_purge` both ways and records what the
/// engine sent it. Its own file because it is a *backend*, not one more provider double, and
/// because the engine's argv decisions are the one subject `MockExecutor` sits too low to reach.
pub mod recording_backend;

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use dashmap::DashMap;
use shall::app::scheduler::{Provisioned, Reading, TaskProvisioner, TaskTarget};
use shall::app::App;
use shall::config::config::ScheduleConfig;
use shall::config::Config;
use shall::core::executor::MockExecutor;
use shall::core::{
    CommandExecutor, Error, PackageSpec, Result, Snapshot, SnapshotProvider, StateRegistry,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct TestKernel {
    pub app: App,
    pub mock_executor: Arc<MockExecutor>,
    pub tmp: TempDir,
    pub state: Arc<Mutex<StateRegistry>>,
    vfs: Arc<DashMap<std::path::PathBuf, String>>,
    lock_map: Arc<DashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

/// Shall no longer injects a commit identity (a signed commit must not be authored by a name
/// nobody owns), so a fixture that commits fails on any host without a global `user.email` —
/// which is every CI runner (S33). The absent config paths keep the developer's own
/// `~/.gitconfig` out too: a host that signs every commit would fail these at `git commit`.
///
/// Twinned in `src/core/git.rs`'s test module, which is a separate binary and cannot link to
/// this one. Change one, change the other.
fn hermetic_git_env() {
    static HERMETIC: std::sync::Once = std::sync::Once::new();
    HERMETIC.call_once(|| {
        for (k, v) in [
            ("GIT_AUTHOR_NAME", "shall-tests"),
            ("GIT_AUTHOR_EMAIL", "test@example.invalid"),
            ("GIT_COMMITTER_NAME", "shall-tests"),
            ("GIT_COMMITTER_EMAIL", "test@example.invalid"),
            ("GIT_CONFIG_GLOBAL", "shall-tests-absent-gitconfig"),
            ("GIT_CONFIG_SYSTEM", "shall-tests-absent-gitconfig"),
        ] {
            std::env::set_var(k, v);
        }
    });
}

impl TestKernel {
    pub async fn new() -> Self {
        // Before anything can commit: set in the kernel, not in the one test that noticed,
        // so a fixture added later cannot forget it.
        hermetic_git_env();

        let tmp = tempfile::Builder::new()
            .prefix("shall_hermetic_")
            .tempdir()
            .expect("Failed to create test sandbox directory.");

        let registry_path = tmp.path().join("registry.json");

        // S11: every path under the sandbox, in one call. Setting the roots by hand here is
        // what let the sibling fixture forget `data_root` and write to real user state.
        // Not a dry run: the executor below is a mock, so nothing reaches the machine, and a
        // fixture that claims preview-only while asserting mutations happened is a fixture
        // that cannot see a preview-only bug (S25). A test that wants the preview path clones
        // this config and sets `dry_run`.
        let config = Config::sandboxed(tmp.path());

        // The II.1 layout, because the resolver now reads a repo rather than a folder of
        // manifests. `priority` says which package managers this machine uses: without it
        // Shall refuses to guess, so a fixture without one is a fixture that cannot resolve
        // anything. These are the three the mock executor pretends to have.
        std::fs::write(tmp.path().join("priority"), "apt\nbrew\ncargo\n")
            .expect("Failed to write test `priority`.");
        std::fs::create_dir_all(tmp.path().join("modules"))
            .expect("Failed to create test `modules/`.");
        std::fs::create_dir_all(tmp.path().join("profiles"))
            .expect("Failed to create test `profiles/`.");
        // A profile has to be active or there is nowhere for a line to go: a module no
        // profile reaches is one Shall never reads, so writing to one is refused rather
        // than done silently. A fixture with nothing active is a fixture that cannot
        // install. Tests that care about a specific profile overwrite these two files.
        std::fs::write(tmp.path().join("profiles/Main"), "")
            .expect("Failed to write test profile.");
        std::fs::write(tmp.path().join("active"), "Main\n")
            .expect("Failed to write test `active`.");

        let vfs = Arc::new(DashMap::new());
        let lock_map = Arc::new(DashMap::new());

        let mock_executor = Arc::new(MockExecutor::new(vfs.clone()));
        // **Every manager the suite probes, seeded once.**
        //
        // `check_command` answers `true` for an unseeded name and records the probe, so a fixture
        // that seeded three left ~59 names falling through on every test that asked. Those probes
        // are not defects — `true` is deliberately what an unopinionated test means — but leaving
        // them unanswered made the fall-through ledger unreadable: 134 of the 352 tests that
        // reported at least one unanswered command reported *only* probes, which is noise in the
        // exact measurement that says which tests are asserting against an empty string they never
        // wrote (`#40`).
        //
        // Seeding here states the assumption the tests were already making. It changes what the
        // ledger holds, not what any verdict is: `check_command`'s default was `true` and this is
        // still `true`. Re-measured after `V.239` — see that entry for the counts.
        mock_executor.set_command_exists("apk", true);
        mock_executor.set_command_exists("apt", true);
        mock_executor.set_command_exists("asdf", true);
        mock_executor.set_command_exists("brew", true);
        mock_executor.set_command_exists("btrfs", true);
        mock_executor.set_command_exists("bun", true);
        mock_executor.set_command_exists("cabal", true);
        mock_executor.set_command_exists("cargo", true);
        mock_executor.set_command_exists("code", true);
        mock_executor.set_command_exists("composer", true);
        mock_executor.set_command_exists("conda", true);
        mock_executor.set_command_exists("dart", true);
        mock_executor.set_command_exists("dkms", true);
        mock_executor.set_command_exists("dnf", true);
        mock_executor.set_command_exists("dotnet", true);
        mock_executor.set_command_exists("dpkg", true);
        mock_executor.set_command_exists("emacs", true);
        mock_executor.set_command_exists("emerge", true);
        mock_executor.set_command_exists("eopkg", true);
        mock_executor.set_command_exists("flatpak", true);
        mock_executor.set_command_exists("gem", true);
        mock_executor.set_command_exists("go", true);
        mock_executor.set_command_exists("gsettings", true);
        mock_executor.set_command_exists("guix", true);
        mock_executor.set_command_exists("helm", true);
        mock_executor.set_command_exists("kubectl", true);
        mock_executor.set_command_exists("kubectl-krew", true);
        mock_executor.set_command_exists("luarocks", true);
        mock_executor.set_command_exists("lvs", true);
        mock_executor.set_command_exists("mise", true);
        mock_executor.set_command_exists("mix", true);
        mock_executor.set_command_exists("moss", true);
        mock_executor.set_command_exists("nimble", true);
        mock_executor.set_command_exists("nix", true);
        mock_executor.set_command_exists("npm", true);
        mock_executor.set_command_exists("opam", true);
        mock_executor.set_command_exists("pacman", true);
        mock_executor.set_command_exists("paru", true);
        mock_executor.set_command_exists("pip", true);
        mock_executor.set_command_exists("pipx", true);
        mock_executor.set_command_exists("pixi", true);
        mock_executor.set_command_exists("pnpm", true);
        mock_executor.set_command_exists("probe-detect", true);
        mock_executor.set_command_exists("pwsh", true);
        mock_executor.set_command_exists("rpm", true);
        mock_executor.set_command_exists("rustup", true);
        mock_executor.set_command_exists("slackpkg", true);
        mock_executor.set_command_exists("snap", true);
        mock_executor.set_command_exists("spack", true);
        mock_executor.set_command_exists("stack", true);
        mock_executor.set_command_exists("systemctl", true);
        mock_executor.set_command_exists("testinit-detect", true);
        mock_executor.set_command_exists("timeshift", true);
        mock_executor.set_command_exists("uv", true);
        mock_executor.set_command_exists("xbps-install", true);
        mock_executor.set_command_exists("yarn", true);
        mock_executor.set_command_exists("yay", true);
        mock_executor.set_command_exists("zfs", true);
        mock_executor.set_command_exists("zypper", true);

        let executor = CommandExecutor::with_layer(
            true,
            false,
            mock_executor.clone(),
            vfs.clone(),
            lock_map.clone(),
        );

        let app = App::new_with_executor_and_state_path(config, executor, Some(registry_path))
            .await
            .expect("Failed to bootstrap Test Kernel.");

        let state = app.state.clone();

        Self {
            app,
            mock_executor,
            tmp,
            state,
            vfs,
            lock_map,
        }
    }

    /// A **second run** over this same sandbox: a fresh `App`, a fresh executor, the same
    /// files, the same state registry, and the same mock machine underneath.
    ///
    /// The executor is rebuilt rather than duplicated because `CommandExecutor::duplicate` is
    /// `clone`, and the installed-listing memo is an `Arc` inside it — so a second sync taken
    /// from a duplicate answers "is it installed?" from the first sync's answer, which is the
    /// one question a convergence test exists to ask twice. The `vfs` and `lock_map` are
    /// shared, because those are the machine.
    #[allow(dead_code)]
    pub async fn second_run(&self) -> App {
        let executor = CommandExecutor::with_layer(
            true,
            false,
            self.mock_executor.clone(),
            self.vfs.clone(),
            self.lock_map.clone(),
        );
        App::new_with_executor_and_state_path(
            (*self.app.config).clone(),
            executor,
            Some(self.tmp.path().join("registry.json")),
        )
        .await
        .expect("a second run bootstraps over the same files")
    }

    /// The same sandbox, previewed: an `App` over these exact files with `--dry-run` set.
    ///
    /// A second `App` rather than a mutated one, because `config` is shared by `Arc` and
    /// flipping the flag in place would flip it for everything already holding a clone.
    #[allow(dead_code)]
    pub async fn previewing(&self) -> App {
        let mut config = (*self.app.config).clone();
        config.dry_run = true;
        App::new_with_executor_and_state_path(
            config,
            self.app.executor.clone(),
            Some(self.tmp.path().join("registry.json")),
        )
        .await
        .expect("Failed to bootstrap the previewing Test Kernel.")
    }

    #[allow(dead_code)]
    pub async fn assert_called(&self, command_fragment: &str) {
        let calls: Vec<String> = self.mock_executor.get_calls().await;
        assert!(
            calls.iter().any(|c| c.contains(command_fragment)),
            "\nCOMMAND VERIFICATION FAILED\nExpected Fragment: '{}'\nActual Call Log:   {:?}\n",
            command_fragment,
            calls
        );
    }
}

#[allow(dead_code)]
pub struct MockSnapshotProvider {
    pub store: Arc<Mutex<Vec<Snapshot>>>,
    pub deletions: Arc<Mutex<Vec<String>>>,
}

#[allow(dead_code)]
impl Default for MockSnapshotProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSnapshotProvider {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(Vec::new())),
            deletions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_historical_snapshot(&self, id: &str, days_ago: i64) {
        let timestamp = Utc::now() - ChronoDuration::days(days_ago);
        let mut list = self.store.lock().await;
        list.push(Snapshot {
            id: id.to_string(),
            timestamp: timestamp.to_rfc3339(),
            description: format!("Historical state from {} days ago", days_ago),
            backend: "mock".into(),
        });
    }
}

#[async_trait]
impl SnapshotProvider for MockSnapshotProvider {
    fn name(&self) -> &str {
        "mock"
    }
    async fn is_available(&self) -> bool {
        true
    }
    async fn create(&self, label: shall::core::snapshot::SnapshotLabel) -> Result<Snapshot> {
        let s = Snapshot {
            id: format!("snap_{}", Utc::now().timestamp()),
            timestamp: Utc::now().to_rfc3339(),
            description: label.to_string(),
            backend: "mock".into(),
        };
        self.store.lock().await.push(s.clone());
        Ok(s)
    }
    async fn list(&self) -> Result<Vec<Snapshot>> {
        Ok(self.store.lock().await.clone())
    }
    async fn delete(&self, id: &str) -> Result<()> {
        let mut list = self.store.lock().await;
        if let Some(pos) = list.iter().position(|s| s.id == id) {
            list.remove(pos);
            self.deletions.lock().await.push(id.to_string());
            Ok(())
        } else {
            Err(Error::Snapshot(format!(
                "Mock Logic Failure: Snapshot {} missing.",
                id
            )))
        }
    }
    fn restore_capability(&self) -> shall::core::snapshot::RestoreCapability {
        shall::core::snapshot::RestoreCapability::Live
    }
    async fn restore(&self, _id: &str) -> Result<()> {
        Ok(())
    }
}

#[allow(dead_code)]
pub struct MockTaskProvisioner {
    pub active_tasks: Arc<Mutex<HashMap<String, ScheduleConfig>>>,
}

#[allow(dead_code)]
impl Default for MockTaskProvisioner {
    fn default() -> Self {
        Self::new()
    }
}

impl MockTaskProvisioner {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn account_marker(target: &TaskTarget) -> String {
    match &target.account {
        None => String::new(),
        Some(account) => format!(" @user={}", account.name()),
    }
}

pub fn mock_task_key(name: &str, target: &TaskTarget) -> String {
    match &target.account {
        None => name.to_string(),
        Some(account) => format!("{name}@user={}", account.name()),
    }
}

#[async_trait]
impl TaskProvisioner for MockTaskProvisioner {
    async fn add_task(
        &self,
        _executor: &CommandExecutor,
        config: &ScheduleConfig,
        _shall_path: &Path,
        target: &TaskTarget,
    ) -> Result<()> {
        let mut map = self.active_tasks.lock().await;
        map.insert(mock_task_key(&config.name, target), config.clone());
        Ok(())
    }
    async fn remove_task(
        &self,
        _executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Result<()> {
        let mut map = self.active_tasks.lock().await;
        map.remove(&mock_task_key(name, target));
        Ok(())
    }
    async fn is_task_active(
        &self,
        _executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> bool {
        let map = self.active_tasks.lock().await;
        map.contains_key(&mock_task_key(name, target))
    }
    /// The mock scheduler expresses everything, so nothing is refused here. A refusal belongs
    /// to the OS that cannot hold the option, and inventing one for the mock would make a test
    /// about the mock's imagination.
    fn refuse_unsupported(&self, _config: &ScheduleConfig) -> std::result::Result<(), String> {
        Ok(())
    }
    /// It stores the whole declaration, so what it would provision and what it holds are the
    /// same value — which is what makes it a mock and not a second implementation.
    fn rendered(
        &self,
        config: &ScheduleConfig,
        _shall_bin: &Path,
        target: &TaskTarget,
    ) -> std::result::Result<Provisioned, String> {
        Ok(Provisioned {
            spec: format!("{:?}{}", config, account_marker(target)),
            armed: config.enabled.unwrap_or(true),
        })
    }
    async fn read_task(
        &self,
        _executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Reading {
        let map = self.active_tasks.lock().await;
        match map.get(&mock_task_key(name, target)) {
            Some(config) => Reading::Holds(Provisioned {
                spec: format!("{:?}{}", config, account_marker(target)),
                armed: config.enabled.unwrap_or(true),
            }),
            None => Reading::Absent,
        }
    }
}

#[allow(dead_code)]
pub fn create_dummy_spec(name: &str, backend: &str, source: Option<&str>) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    if let Some(src) = source {
        options.set("__source", src);
    }
    PackageSpec {
        name: name.to_string(),
        backend: backend.to_string(),
        options,
        requires: Vec::new(),
        present: true,
    }
}
