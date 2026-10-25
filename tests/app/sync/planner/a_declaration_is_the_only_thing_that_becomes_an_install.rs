use super::*;

/// **A plan installs what you declared. Not what your declarations depend on.**
///
/// Every install node is written into the state registry as a package Shall manages, and
/// anything in that registry is a removal candidate the moment nothing declares it. A
/// dependency is never declared, so expanding one manufactures a managed package with no
/// line behind it — `Queryable::tracks_manual` says exactly this about `adopt`, which
/// refuses a backend that cannot tell a dependency from a choice, and the planner was
/// doing by construction what `adopt` refuses to do.
#[tokio::test]
async fn a_declaration_is_the_only_thing_that_becomes_an_install() {
    let backend = DepAnswering::new(&[("nginx", &["libfoo", "libbar"])]);
    let asked = backend.asked.clone();
    let registry = dep_registry(backend);
    let config = Config::default();
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> =
        [("deptest".to_string(), vec![declared("nginx")])]
            .into_iter()
            .collect();

    let changes = ChangePlanner::new(registry, &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();

    let report = changes.generate_report();
    let names: Vec<&str> = report.install.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["nginx"],
        "libfoo and libbar are the manager's business, not Shall's"
    );
    assert_eq!(
        asked.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "planning must not spend a subprocess asking what a package depends on"
    );
}
