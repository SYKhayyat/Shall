use shall::app::sync::planner::{ChangePlanner, HostBackends, PlanScope, Scope};
use shall::app::sync::resolver::StateResolver;
use shall::core::LockFile;
use shall::core::{Error, PackageSpec};
use std::collections::HashMap;
use tokio::fs;

// Import our exhaustive A+ Test Infrastructure
use crate::mock_providers::TestKernel;

// ============================================================================
// MODULES: `use` and recursive expansion
// ============================================================================

/// A module `use`ing a module, reached by a profile, expands to a flat closure.
///
/// 1. `use NAME` takes a name — never a path, never a URL.
/// 2. Deep nesting resolves without cycles.
/// 3. Each package records where it came from and what it belongs to.
#[tokio::test]
async fn test_recursive_module_expansion_logic() {
    let kernel = TestKernel::new().await;
    let root = kernel.app.config.config_root();

    // 1. The leaf module.
    fs::write(root.join("modules/network.txt"), "brew:curl\nbrew:wget\n")
        .await
        .unwrap();

    // 2. A module that uses it. A module may use a module; it may never name a profile.
    fs::write(root.join("modules/bundle.txt"), "use network\nbrew:git\n")
        .await
        .unwrap();

    // 3. A profile to reach it, and the machine set to that profile. Only profiles can be
    //    activated, and nothing is active unless a profile names it.
    fs::write(root.join("profiles/Work"), "use bundle\n")
        .await
        .unwrap();
    fs::write(root.join("active"), "Work\n").await.unwrap();

    let resolver = StateResolver::new(&kernel.app.config, kernel.app.registry.clone(), false).await;
    let desired = resolver
        .resolve_desired_state()
        .await
        .expect("State resolution failed");

    let brew_specs = desired
        .get("brew")
        .expect("Missing brew backend specs in resolution map");
    let names: Vec<&str> = brew_specs.iter().map(|s| s.name.as_str()).collect();

    assert!(
        names.contains(&"curl"),
        "Resolver failed to expand nested leaf 'curl' from `network`"
    );
    assert!(
        names.contains(&"wget"),
        "Resolver failed to expand nested leaf 'wget' from `network`"
    );
    assert!(
        names.contains(&"git"),
        "Resolver failed to expand direct member 'git' from `bundle`"
    );
    assert_eq!(
        names.len(),
        3,
        "Expanded closure count mismatch. Expected 3 packages."
    );

    // Where the line is, for a human; and what it belongs to, for `--module` / `--profile`.
    let curl_spec = brew_specs.iter().find(|s| s.name == "curl").unwrap();
    assert!(curl_spec
        .options
        .one("__source")
        .unwrap()
        .contains("network.txt:1"));
    // `__scopes` is a list, so membership is a list question. It was a `;`-joined string that
    // every reader split for itself, and `contains` on it could not tell `module:network` from
    // `module:network-extras`.
    let scopes = curl_spec.options.all("__scopes");
    assert!(scopes.iter().any(|s| s == "module:network"), "{scopes:?}");
    assert!(scopes.iter().any(|s| s == "profile:Work"), "{scopes:?}");
}

/// W13: the plan explains a variable-driven change by diffing this run's variables against the
/// last successful sync (HEAD). This proves the baseline half — `vars_at_last_sync` reads the
/// committed `vars`, `resolve_vars` reads the working tree, and `vars::diff` names the change.
#[tokio::test]
async fn vars_change_is_measured_against_the_committed_baseline() {
    let kernel = TestKernel::new().await;
    let root = kernel.app.config.config_root().to_path_buf();

    fs::write(root.join("vars"), "role = travel\n")
        .await
        .unwrap();
    let git = kernel.app.vcs().manager();
    git.init().unwrap();
    git.commit_all("baseline").unwrap();

    // Edit the working tree without committing — this is the "you edited vars" state.
    fs::write(root.join("vars"), "role = desktop\n")
        .await
        .unwrap();

    let resolver = StateResolver::new(&kernel.app.config, kernel.app.registry.clone(), false).await;
    let baseline = resolver
        .vars_at_last_sync(&git)
        .await
        .unwrap()
        .expect("HEAD has a vars file, so there is a baseline");
    let now = resolver.resolve_vars().await.unwrap();

    assert_eq!(
        baseline["role"],
        shall::model::vars::Value::Str("travel".into())
    );
    assert_eq!(
        now["role"],
        shall::model::vars::Value::Str("desktop".into())
    );

    let changed = shall::model::vars::diff(&baseline, &now);
    assert_eq!(changed.len(), 1, "only role changed: {:?}", changed);
    assert_eq!(changed[0].0, "role");
}

/// A `use` of a module that does not exist is a descriptive error, never a silent skip and
/// never a package named `ghost-module-123`.
#[tokio::test]
async fn test_module_resolution_failure_handling() {
    let kernel = TestKernel::new().await;
    let root = kernel.app.config.config_root();

    fs::write(root.join("profiles/Work"), "use ghost-module-123\n")
        .await
        .unwrap();
    fs::write(root.join("active"), "Work\n").await.unwrap();

    let resolver = StateResolver::new(&kernel.app.config, kernel.app.registry.clone(), false).await;
    let result = resolver.resolve_desired_state().await;

    assert!(
        result.is_err(),
        "Resolution should have failed for a missing module reference"
    );
    if let Err(Error::Config(msg)) = result {
        assert!(
            msg.contains("ghost-module-123"),
            "Error message should identify the specific missing module: {}",
            msg
        );
    } else {
        panic!("Incorrect error type returned: {:?}", result.err());
    }
}

// ============================================================================
// FEATURE 1: STRUCTURED JSON DRY-RUN REPORTING
// ============================================================================

/// Verifies that the ChangePlanner generates a SyncReport with accurate fields
/// suitable for high-fidelity JSON serialization.
#[tokio::test]
async fn test_sync_report_generation_schema_fidelity() {
    let kernel = TestKernel::new().await;
    let state_guard = kernel.state.lock().await;

    let planner = ChangePlanner::new(
        kernel.app.registry.clone(),
        &state_guard,
        &kernel.app.config,
    );

    // Seed a desired state containing source metadata
    let mut desired = HashMap::new();
    desired.insert(
        "brew".to_string(),
        vec![PackageSpec {
            name: "ripgrep".into(),
            backend: "brew".into(),
            options: {
                let mut o = shall::config::grammar::Options::default();
                o.insert("__source".to_string(), "module:dev-tools");
                o
            },
            requires: vec![],
            present: true,
        }],
    );

    // Plan with None (Global Sync)
    let plan = planner
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();
    let report = plan.generate_report();

    // 1. Verify business logic mapping to the Report structure
    assert_eq!(report.change_count, 1);
    assert_eq!(report.install.len(), 1);
    assert_eq!(report.install[0].name, "ripgrep");
    assert_eq!(
        report.install[0].source,
        Some("module:dev-tools".to_string())
    );

    // 2. Verify JSON Serialization (Schema Integrity)
    let json_output = serde_json::to_string(&report).expect("SyncReport failed JSON serialization");
    assert!(
        json_output.contains("\"change_count\":1"),
        "JSON missing change_count field"
    );
    assert!(
        json_output.contains("\"name\":\"ripgrep\""),
        "JSON missing package name"
    );
    assert!(
        json_output.contains("\"source\":\"module:dev-tools\""),
        "JSON missing source metadata"
    );
}

// ============================================================================
// FEATURE 4: SCOPED UPGRADE LOGIC
// ============================================================================

/// Verifies that a Scope correctly prunes the DAG to only include
/// nodes matching the requested source origin.
#[tokio::test]
async fn test_scoped_planner_filtering_accuracy() {
    let kernel = TestKernel::new().await;
    let state_guard = kernel.state.lock().await;
    let planner = ChangePlanner::new(
        kernel.app.registry.clone(),
        &state_guard,
        &kernel.app.config,
    );

    // Setup a mixed desired state. A package belongs to the module that holds it and the
    // profile that reaches it, and the resolver records both.
    let mut desired = HashMap::new();
    desired.insert(
        "brew".to_string(),
        vec![
            PackageSpec {
                name: "pkg-work".into(),
                backend: "brew".into(),
                options: {
                    let mut o = shall::config::grammar::Options::default();
                    o.insert("__scopes".to_string(), "module:dev");
                    o.insert("__scopes".to_string(), "profile:Work");
                    o
                },
                requires: vec![],
                present: true,
            },
            PackageSpec {
                name: "pkg-home".into(),
                backend: "brew".into(),
                options: {
                    let mut o = shall::config::grammar::Options::default();
                    o.insert("__scopes".to_string(), "module:media");
                    o.insert("__scopes".to_string(), "profile:Home");
                    o
                },
                requires: vec![],
                present: true,
            },
        ],
    );

    let plan = planner
        .plan(&desired, PlanScope::Narrowed(Scope::Profile("Work".into())))
        .await
        .unwrap();

    // Verification
    assert_eq!(
        plan.total_install(),
        1,
        "Planner failed to prune the graph based on scope"
    );
    let report = plan.generate_report();
    assert_eq!(report.install[0].name, "pkg-work");
    assert!(
        !report.install.iter().any(|r| r.name == "pkg-home"),
        "Package from outside the scope (profile Home) was incorrectly included in the plan"
    );
}

// ============================================================================
// BUG FIX 3: --LOCKED MODE INTEGRITY
// ============================================================================

/// Verifies that Locked Mode prevents resolution if manifest versions deviate
/// from the cryptographically tracked locks.json.
#[tokio::test]
async fn test_locked_mode_version_conflict_enforcement() {
    let kernel = TestKernel::new().await;

    // 1. Setup a lock file with version 1.2.3
    let lock_content = r#"{ "locks": { "brew:vim": "1.2.3" } }"#;
    fs::create_dir_all(kernel.app.config.config_root().join("locks"))
        .await
        .unwrap();
    fs::write(
        kernel
            .app
            .config
            .config_root()
            .join("locks")
            .join("versions.json"),
        lock_content,
    )
    .await
    .unwrap();

    // 2. A module requesting a conflicting version 2.0.0, and a profile reaching it.
    let root = kernel.app.config.config_root();
    fs::write(root.join("modules/main.txt"), "brew:vim@version=2.0.0\n")
        .await
        .unwrap();
    fs::write(root.join("profiles/Work"), "use main\n")
        .await
        .unwrap();
    fs::write(root.join("active"), "Work\n").await.unwrap();

    // 3. Resolve in Locked Mode (locked = true)
    let resolver = StateResolver::new(&kernel.app.config, kernel.app.registry.clone(), true).await;
    let result = resolver.resolve_desired_state().await;

    // 4. Assert Failure
    assert!(
        result.is_err(),
        "Resolver should have rejected the version mismatch in locked mode"
    );
    if let Err(Error::Validation(msg)) = result {
        assert!(
            msg.contains("version mismatch"),
            "Incorrect validation error message: {}",
            msg
        );
        assert!(
            msg.contains("brew:vim"),
            "Error should identify the offending package"
        );
    }
}

// ============================================================================
// EXTRAS: the teardown ledger
// ============================================================================

/// An undo that fails must not be forgotten. `reconcile` records what is declared now,
/// so a key whose teardown failed used to vanish from `locks/extras.toml` after one warning —
/// leaving a service or a timer in place that Shall no longer knows it owns. It stays recorded
/// until the undo succeeds.
#[tokio::test]
async fn a_failed_undo_stays_in_the_extras_ledger() {
    let kernel = TestKernel::new().await;
    let locks = kernel.app.config.config_root().join("locks");
    let path = shall::core::ExtrasLedger::path_in(&locks);

    // `no-such-backend` cannot be resolved, so its teardown fails for a reason no host can
    // fix by luck. Nothing declares it, so it is drift the moment the ledger is read.
    let mut ledger = shall::core::ExtrasLedger::new();
    ledger.record(vec![row("repo:no-such-backend:ppa/example")]);
    ledger.save(&path).unwrap();

    let state = shall::model::DesiredState::default();
    kernel
        .app
        .extras()
        .reconcile(&state, shall::app::sync::guard::GuardScope::Sync)
        .await
        .expect("a failed undo is reported, not fatal");

    let after = shall::core::ExtrasLedger::load(&path).unwrap();
    let wires: Vec<String> = after.records().iter().map(|r| r.wire()).collect();
    assert!(
        wires.contains(&"repo:no-such-backend:ppa/example".to_string()),
        "the failed teardown was dropped from the ledger: {wires:?}"
    );
}

/// And the other half: a teardown that succeeds does leave the ledger, or every sync would
/// retry an undo forever.
#[tokio::test]
async fn a_successful_undo_leaves_the_extras_ledger() {
    let kernel = TestKernel::new().await;
    let locks = kernel.app.config.config_root().join("locks");
    let path = shall::core::ExtrasLedger::path_in(&locks);

    // A `shim:` nobody deployed: `remove_shim` finds nothing to delete and returns `Ok(())`,
    // which is a genuinely successful teardown on any host and needs no service manager.
    //
    // **This used to plant `nosuchkind:whatever`**, under a comment explaining that an unknown
    // kind "warns and reports success". That was `undo_extra`'s catch-all, and reporting success
    // there dropped the row from the ledger while the resource stayed on the machine (`S56`) —
    // so this test was pinning the bug as the expected behaviour, which is exactly the shape
    // `S16` records. An unreadable row is now kept, and the test needed a real success.
    let mut ledger = shall::core::ExtrasLedger::new();
    ledger.record(vec![row("shim:no-such-shim")]);
    ledger.save(&path).unwrap();

    let state = shall::model::DesiredState::default();
    kernel
        .app
        .extras()
        .reconcile(&state, shall::app::sync::guard::GuardScope::Sync)
        .await
        .unwrap();

    let after = shall::core::ExtrasLedger::load(&path).unwrap();
    assert!(
        after.is_empty(),
        "a successful teardown was left recorded: {:?}",
        after.records()
    );
}

use crate::harness::{decl, Fixture};
use shall::core::extras_lock::DirOrigin;

fn row(wire: &str) -> shall::core::extras_lock::ExtraRecord {
    shall::core::extras_lock::ExtraRecord::parse(wire).expect("a well-formed row")
}

fn ledger_body(f: &Fixture) -> String {
    std::fs::read_to_string(f.cfg().join("locks").join("extras.toml"))
        .expect("a sync that placed a resource records it")
}

fn origin_of(body: &str, subject: &str) -> Option<DirOrigin> {
    let wanted = format!("subject = {subject:?}");
    body.lines()
        .find(|line| *line == wanted)
        .and_then(|_| {
            body.lines()
                .skip_while(|line| *line != wanted)
                .find_map(|line| line.strip_prefix("created = "))
        })
        .map(|spelled| match spelled.trim_matches('"') {
            "shall-created" => DirOrigin::ShallCreated,
            "pre-existing" => DirOrigin::PreExisting,
            other => panic!("`{other}` is not an origin this build knows"),
        })
}

#[test]
fn a_directory_shall_created_is_recorded_as_its_own_and_removed_on_undeclare() {
    let f = Fixture::new("extras-dir-shall-created");
    let dir = f.root.join("shall-made");
    f.write_module(&format!("dir:{}\n", decl(&dir)));

    let (out, code) = f.run(&["sync", "-y"]);
    assert_eq!(code, 0, "the first sync failed:\n{out}");
    assert!(dir.is_dir(), "setup did not create {}", dir.display());

    let body = ledger_body(&f);
    assert!(
        body.contains("schema = 2"),
        "the ledger is not written to the schema this build reads:\n{body}"
    );
    assert_eq!(
        origin_of(&body, &dir.display().to_string()),
        Some(DirOrigin::ShallCreated),
        "a directory Shall made was not recorded as Shall's, so nothing on undeclare would be \
         authorized to remove it:\n{body}"
    );

    f.write_module("");
    let (out, code) = f.run(&["sync", "-y"]);
    assert_eq!(code, 0, "the undeclaring sync failed:\n{out}");
    assert!(
        !dir.exists(),
        "{} was Shall's own and was left behind",
        dir.display()
    );
    assert!(
        !ledger_body(&f).contains("subject = "),
        "a successful teardown left its row recorded"
    );
}

#[test]
fn a_pre_existing_empty_directory_is_recorded_as_the_users_and_preserved() {
    let f = Fixture::new("extras-dir-pre-existing");
    let dir = f.root.join("already-there");
    std::fs::create_dir(&dir).unwrap();
    f.write_module(&format!("dir:{}\n", decl(&dir)));

    let (out, code) = f.run(&["sync", "-y"]);
    assert_eq!(code, 0, "the first sync failed:\n{out}");

    let body = ledger_body(&f);
    assert_eq!(
        origin_of(&body, &dir.display().to_string()),
        Some(DirOrigin::PreExisting),
        "a directory that was already on the machine was recorded as Shall's own:\n{body}"
    );

    f.write_module("");
    let (out, code) = f.run(&["sync", "-y"]);
    assert_eq!(code, 0, "the undeclaring sync failed:\n{out}");
    assert!(
        dir.is_dir(),
        "{} existed before the line named it, and was deleted by the line leaving",
        dir.display()
    );
    assert!(
        !ledger_body(&f).contains("subject = "),
        "a preserved directory is no longer declared and must not be retried for ever"
    );
}

#[test]
fn a_shall_created_directory_with_contents_is_preserved_and_its_row_kept() {
    let f = Fixture::new("extras-dir-in-use");
    let dir = f.root.join("in-use");
    f.write_module(&format!("dir:{}\n", decl(&dir)));

    let (out, code) = f.run(&["sync", "-y"]);
    assert_eq!(code, 0, "the first sync failed:\n{out}");
    assert!(dir.is_dir(), "setup did not create {}", dir.display());
    std::fs::write(dir.join("the-users-data"), "keep me\n").unwrap();

    f.write_module("");
    let (out, code) = f.run(&["sync", "-y"]);
    assert_eq!(code, 0, "the undeclaring sync failed:\n{out}");

    assert!(
        dir.join("the-users-data").exists(),
        "a non-empty directory had its contents destroyed by a teardown"
    );
    let body = ledger_body(&f);
    assert_eq!(
        origin_of(&body, &dir.display().to_string()),
        Some(DirOrigin::ShallCreated),
        "a teardown that left the directory in place must keep its row, or the next sync has \
         forgotten that Shall owns the path:\n{body}"
    );
}

#[test]
fn two_directories_keep_separate_rows_and_separate_origins() {
    let f = Fixture::new("extras-dir-two");
    let made = f.root.join("one");
    let found = f.root.join("two");
    std::fs::create_dir(&found).unwrap();
    f.write_module(&format!("dir:{}\ndir:{}\n", decl(&made), decl(&found)));

    let (out, code) = f.run(&["sync", "-y"]);
    assert_eq!(code, 0, "the first sync failed:\n{out}");
    assert!(made.is_dir(), "setup did not create {}", made.display());

    let body = ledger_body(&f);
    assert_ne!(made, found, "two directories wrote one row");
    assert_eq!(
        origin_of(&body, &made.display().to_string()),
        Some(DirOrigin::ShallCreated)
    );
    assert_eq!(
        origin_of(&body, &found.display().to_string()),
        Some(DirOrigin::PreExisting)
    );

    f.write_module("");
    let (out, code) = f.run(&["sync", "-y"]);
    assert_eq!(code, 0, "the undeclaring sync failed:\n{out}");
    assert!(!made.exists(), "{} was Shall's own", made.display());
    assert!(
        found.is_dir(),
        "{} was not Shall's to remove",
        found.display()
    );
}

#[test]
fn a_ledger_in_the_old_format_is_refused_and_an_absent_one_is_empty() {
    use shall::core::LockFile;

    let f = Fixture::bare("extras-ledger-format");
    let path = shall::core::ExtrasLedger::path_in(&f.cfg().join("locks"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    assert!(
        shall::core::ExtrasLedger::load(&path).unwrap().is_empty(),
        "a machine that has never synced has an empty ledger, not an error"
    );

    std::fs::write(&path, "applied = [\"shim:rg\"]\n").unwrap();
    let err = shall::core::ExtrasLedger::load(&path).expect_err("the old format is refused");
    let err = err.to_string();
    assert!(err.contains("schema-1"), "{err}");
    assert!(
        err.contains("shall sync -y"),
        "the refusal must say how to start a fresh one: {err}"
    );
    assert!(
        path.exists(),
        "the refusal must not have rewritten the file"
    );
}

#[test]
fn no_other_kind_of_row_carries_a_creation_origin() {
    for wire in [
        "shim:rg",
        "service:nginx",
        "link:/home/u/.vimrc",
        "setting:org.gnome.x/theme@user=alice",
    ] {
        assert!(
            row(wire)
                .with_created(Some(DirOrigin::ShallCreated))
                .is_err(),
            "{wire} accepted a creation origin"
        );
        assert!(
            row(wire).owns_removal(),
            "{wire}: a teardown that removes a declaration is not gated on provenance"
        );
    }
    let dir = row("dir:/var/lib/shall")
        .with_created(Some(DirOrigin::ShallCreated))
        .unwrap();
    assert!(dir.owns_removal());
    assert!(!row("dir:/home/u/state").owns_removal());
}
