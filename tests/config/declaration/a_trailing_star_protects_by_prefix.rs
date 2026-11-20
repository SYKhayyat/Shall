use super::*;

#[test]
fn a_trailing_star_protects_by_prefix() {
    // The accessor was documented as "EXACT only" while the code has always honoured
    // `*`, and the shipped defaults rely on it (`libperl*`). Believing the doc means
    // hand-expanding the wildcard and losing protection for whatever the list misses.
    let cfg = Config {
        guard: GuardSettings {
            protected_packages: vec!["libperl*".into(), "libc".into()],
            ..Default::default()
        },
        ..Config::default()
    };
    assert!(cfg.protection_rule("libperl5.38t64").is_some());
    assert!(cfg.protection_rule("LIBPERL-BASE").is_some());
    // A bare entry stays exact: the wildcard has to be asked for.
    assert!(cfg.protection_rule("libc-bin").is_none());
}
