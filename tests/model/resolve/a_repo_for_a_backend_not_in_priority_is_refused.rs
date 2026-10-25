use super::*;

#[test]
fn a_repo_for_a_backend_not_in_priority_is_refused() {
    // V.47/V.15. `snap` is not in this fixture's priority (apt, cargo).
    let f = fx(
        "Work\n",
        &[("Work", "use py\n")],
        &[("py.txt", "repo:snap:whatever\n")],
    );
    let err = resolve(&f).unwrap_err();
    assert!(err.what.contains("isn't in your priority list"), "{}", err);
}
