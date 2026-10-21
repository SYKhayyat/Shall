use super::*;

/// The way past `max_total_changes` is **either** flag, and the refusal that blocks a run
/// has to say so.
///
/// It named `--allow-mass-removal` alone, which is the more expensive half of `J9`: this is
/// the instruction someone reads while blocked, so a pure-install run was told that the way
/// to get its installs through was to authorize mass deletion. `shall protected` has printed
/// the true rule all along â€” the guard's own refusal was the surface contradicting it.
#[test]
fn the_total_ceiling_refusal_offers_both_flags() {
    let mut cfg = config_with(10);
    cfg.guard.max_total_changes = 2;
    let err = enforce_total(&cfg, 3, &Reaping::new(), GuardScope::Sync, "install")
        .expect_err("three changes over a total of two is a refusal");
    let err = err.to_string();
    assert!(err.contains("--allow-mass-removal"), "{err}");
    assert!(
        err.contains("--allow-mass-install"),
        "either flag answers this ceiling, and the blocked run has to be told: {err}"
    );

    // The per-kind ceilings are *not* siblings of this: `max_removals`,
    // `max_extra_removals` and `max_port_closures` answer to `--allow-mass-removal` alone
    // (`Y20`), so offering the install flag there would name one that does not work.
    let report = GuardReport {
        objections: vec![Objection::TooMany {
            count: 40,
            limit: 10,
            setting: "max_removals",
        }],
        allowed_by_flag: Vec::new(),
    };
    let refusal = report.message(GuardScope::Sync, RemovalKind::Package);
    assert!(refusal.contains("--allow-mass-removal"), "{refusal}");
    assert!(
        !refusal.contains("--allow-mass-install"),
        "a removal ceiling does not answer to it: {refusal}"
    );
}
