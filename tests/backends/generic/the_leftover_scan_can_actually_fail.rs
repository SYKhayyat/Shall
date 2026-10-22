use super::*;

/// **The guard's own self-test.** The `Debug` scan exists because `walk_args` is a
/// hand-written list, and a check that cannot fail is worse than no check — so this plants
/// exactly what `walk_args` does not visit (a *binary* name, which is not argv and is
/// deliberately outside the walk) and requires the scan to catch it.
#[test]
fn the_leftover_scan_can_actually_fail() {
    let mut cfg = row("m");
    cfg.list_binary = Some("{setting.tool}".into());
    let err = cfg
        .resolve_settings(Some(&settings(&[("tool", "dpkg-query")])))
        .expect_err("the scan must catch a placeholder walk_args does not reach");
    assert!(
        err.to_string().contains("survived resolution"),
        "wrong failure: {err}"
    );
}
