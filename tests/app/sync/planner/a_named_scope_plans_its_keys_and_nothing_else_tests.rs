use super::*;

// `Named` is the scope `C7` ruled: the keys this run may act on. Each test carries a control
// that reads the same input as a whole-machine converge, so "nothing else was planned" cannot
// pass on a plan that planned nothing at all.

fn wanted(name: &str) -> PackageSpec {
    PackageSpec {
        name: name.into(),
        backend: "generic-test".into(),
        ..PackageSpec::default()
    }
}

fn desired_of(specs: &[PackageSpec]) -> HashMap<String, Vec<PackageSpec>> {
    let mut out: HashMap<String, Vec<PackageSpec>> = HashMap::new();
    for s in specs {
        out.entry(s.backend.clone()).or_default().push(s.clone());
    }
    out
}

/// The scope under test.
fn named(keys: &[&str]) -> PlanScope {
    PlanScope::Named(keys.iter().map(|k| (*k).to_string()).collect())
}

/// Two managers, for the move test: the old copy and the new one are on different ones.
fn two_managers() -> Arc<BackendRegistry> {
    let mut registry = BackendRegistry::new();
    register_fake(&mut registry, "generic-test", &[]);
    register_fake(&mut registry, "other-test", &[]);
    Arc::new(registry)
}

/// One plan, over one machine's state, with whatever scope the test is about.
async fn plan(
    registry: Arc<BackendRegistry>,
    state: &StateRegistry,
    desired: &HashMap<String, Vec<PackageSpec>>,
    scope: PlanScope,
) -> SyncChanges {
    ChangePlanner::new(registry, state, &Config::default())
        .plan(desired, scope)
        .await
        .expect("planning must not fail")
}

/// `install X` installs X. It does not install the other thirty lines, which is the whole of
/// `C7` on the install side — and the measured cost of getting it wrong was a native sweep
/// installing `brew:rustup` under the name of `cargo:hexyl`.
#[tokio::test]
async fn a_named_scope_installs_only_its_keys() {
    let registry = registry_reporting("generic-test", &[]);
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired = desired_of(&[wanted("wanted"), wanted("other")]);

    let scoped = plan(
        registry.clone(),
        &state,
        &desired,
        named(&["generic-test:wanted"]),
    )
    .await;
    assert_eq!(
        scoped.total_install(),
        1,
        "a plan naming one package planned {} installs; the other line is not this run's to \
         apply",
        scoped.total_install()
    );

    // The control: the same input read as a whole-machine converge installs both, so the
    // assertion above is measuring the filter and not an empty plan.
    let whole = plan(
        registry,
        &state,
        &desired,
        PlanScope::Whole(HostBackends::default()),
    )
    .await;
    assert_eq!(
        whole.total_install(),
        2,
        "control: a whole-machine converge installs both declarations"
    );
}

/// `uninstall X` removes X, and reaps nothing else. A narrowed run that scheduled removals would
/// make every `install` a mass-removal verb — the shape `sync` owns.
#[tokio::test]
async fn a_named_scope_reaps_only_its_keys() {
    let registry = registry_reporting("generic-test", &[]);
    let mut state = StateRegistry::new(PathBuf::from("test-state.json"));
    state.set_managed([
        managed("wanted", "generic-test"),
        managed("other", "generic-test"),
        managed("third", "generic-test"),
    ]);
    // Nothing declares any of them: the uninstall shape, where the line is gone and the package
    // is still installed.
    let nothing = HashMap::new();

    let scoped = plan(
        registry.clone(),
        &state,
        &nothing,
        named(&["generic-test:wanted"]),
    )
    .await;
    assert_eq!(
        scoped.total_remove(),
        1,
        "a plan naming one package removed {}; the other two are not this run's to remove",
        scoped.total_remove()
    );

    let whole = plan(
        registry,
        &state,
        &nothing,
        PlanScope::Whole(HostBackends::default()),
    )
    .await;
    assert_eq!(
        whole.total_remove(),
        3,
        "control: a whole-machine converge reaps all three"
    );
}

/// The `absent:` half of the same two loops. `uninstall X --absent` writes one `absent:` line
/// and enforces that one, not every absence the config holds.
#[tokio::test]
async fn a_named_scope_acts_on_the_absent_declaration_it_names() {
    let registry = registry_reporting("generic-test", &["gone-a", "gone-b"]);
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> = [(
        "generic-test".to_string(),
        vec![
            absent_spec("gone-a", "generic-test"),
            absent_spec("gone-b", "generic-test"),
        ],
    )]
    .into_iter()
    .collect();

    let scoped = plan(
        registry.clone(),
        &state,
        &desired,
        named(&["generic-test:gone-a"]),
    )
    .await;
    assert_eq!(
        scoped.total_remove(),
        1,
        "only the named absence is enforced; `gone-b` is a line this command did not write"
    );

    let whole = plan(
        registry,
        &state,
        &desired,
        PlanScope::Whole(HostBackends::default()),
    )
    .await;
    assert_eq!(whole.total_remove(), 2, "control: a converge enforces both");
}

/// Naming a package that is already converged schedules nothing. The narrowing is a filter, not
/// a licence to reinstall: an install of something already there is `already up to date`.
#[tokio::test]
async fn a_named_scope_schedules_nothing_for_a_named_package_that_is_present() {
    let registry = registry_reporting("generic-test", &["wanted"]);
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired = desired_of(&[wanted("wanted"), wanted("other")]);

    let scoped = plan(
        registry.clone(),
        &state,
        &desired,
        named(&["generic-test:wanted"]),
    )
    .await;
    assert_eq!(
        scoped.total_install(),
        0,
        "`wanted` is installed, and nothing else is in the question"
    );

    // …and the control proves the machine's answer was read rather than assumed: over the whole
    // config, the one package that is missing is planned.
    let whole = plan(
        registry,
        &state,
        &desired,
        PlanScope::Whole(HostBackends::default()),
    )
    .await;
    assert_eq!(whole.total_install(), 1, "control: `other` is missing");
}

/// `retarget` — one plan installs the new copy and removes the old one, because both keys are
/// named. Naming only the new key removes nothing, which is the difference between a mechanism
/// and an accident.
#[tokio::test]
async fn a_named_scope_carries_one_packages_install_and_removal_together() {
    let registry = two_managers();
    let mut state = StateRegistry::new(PathBuf::from("test-state.json"));
    state.set_managed([managed("moved", "generic-test")]);
    let desired = desired_of(&[wanted("moved")]);

    let both = plan(
        registry.clone(),
        &state,
        &desired,
        named(&["generic-test:moved"]),
    )
    .await;
    // Declared and managed under the same key: the old copy is `StillDeclared`, so this is the
    // one-key case and must be an install only.
    assert_eq!(
        (both.total_install(), both.total_remove()),
        (1, 0),
        "a key that is both declared and managed is an install, not a move"
    );

    // The teleport shape: the line now names another manager, so the old key is drift and the
    // new key is a declaration. Both are in the question, and the answer is one plan with both.
    let mut moved = state.clone();
    moved.set_managed([managed("moved", "other-test")]);
    let teleport = plan(
        registry.clone(),
        &moved,
        &desired,
        named(&["generic-test:moved", "other-test:moved"]),
    )
    .await;
    assert_eq!(
        (teleport.total_install(), teleport.total_remove()),
        (1, 1),
        "the named move installs the new copy and removes the old, in one plan"
    );

    // The control: only the new key, and nothing is removed. Without this, a Named scope that
    // ignored its keys would pass the assertion above.
    let new_only = plan(registry, &moved, &desired, named(&["generic-test:moved"])).await;
    assert_eq!(
        (new_only.total_install(), new_only.total_remove()),
        (1, 0),
        "control: naming only the new key must not remove the old copy"
    );
}
