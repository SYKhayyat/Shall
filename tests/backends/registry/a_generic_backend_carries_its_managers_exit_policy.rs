use super::*;

/// Every manager the exit-policy table knows carries its policy into the registry.
///
/// **An exit policy is not argv, so the argv table cannot see it.** Converting `cargo` and
/// `pipx` from hand-written modules to data on 2026-08-04 dropped their `with_exit_policy`
/// line; every argv assertion stayed green and `cargo install <no-such-crate>` silently
/// stopped being classified `permanent`, which sends the sweep harness back to retrying a
/// crate that will never exist. Two integration tests caught it after the fact. This is the
/// same check one layer down, so the next conversion fails here first.
#[test]
fn a_generic_backend_carries_its_managers_exit_policy() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    // Filtered on the predicate, not on a hand-written list: `helm` has a policy entry
    // that carries benign exit codes and no absent markers, so asserting it "classifies"
    // asserts something untrue about helm rather than something true about the wiring.
    let known: Vec<&str> = argv_cases()
        .iter()
        .map(|c| c.backend)
        .filter(|n| shall::core::exit_policy::classifies_absent_names(n))
        .collect();
    assert!(
        known.len() >= 5,
        "only {} backends classify absent names — the filter is broken, not the code",
        known.len()
    );
    for name in known {
        let vfs = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let exec = CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new()));
        assert!(
            !exec.classifies_absent_names(),
            "the bare executor already classifies, so this test proves nothing"
        );
        let core = Arc::new(GenericBackendCore {
            name: name.to_string(),
            executor: exec,
            config: base_config(name),
            parser: Arc::new(LambdaParser {
                installed_fn: |_| Ok(vec![]),
                search_fn: |_| vec![],
            }),
        });
        assert!(
            with_manager_policy(core).executor.classifies_absent_names(),
            "`{name}` has an entry in exit_policy::for_manager and did not carry it. A \
             manager that cannot say \"no such package\" leaves the line in the manifest \
             and every later command fails on it."
        );
    }
}
