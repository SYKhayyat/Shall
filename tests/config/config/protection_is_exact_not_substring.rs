use super::*;

#[test]
fn protection_is_exact_not_substring() {
    let cfg = Config {
        guard: GuardSettings {
            protected_packages: vec!["libc".into(), "apt".into(), "kernel".into()],
            ..Default::default()
        },
        ..Config::default()
    };
    // exact matches (case-insensitive) are protected
    assert!(cfg.protection_rule("libc").is_some());
    assert!(cfg.protection_rule("APT").is_some());
    // substrings/superstrings are NOT protected (the old bug)
    assert!(cfg.protection_rule("libc-bin").is_none());
    assert!(cfg.protection_rule("aptitude").is_none());
    assert!(cfg.protection_rule("kernelshark").is_none());
}
