use super::*;

#[test]
fn unprotecting_cannot_release_a_name_that_cannot_be_declared() {
    // `unprotected_packages` says "I manage this one myself". You cannot manage what you
    // cannot write down, so this is the one protection the escape hatch does not open.
    let cfg = Config {
        guard: shall::config::GuardSettings {
            unprotected_packages: vec!["*".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    // Quotable now, so `*` really does release it â€” the escape hatch works on every name
    // the user could have written.
    assert!(protection_of(&cfg, Some("winget"), "Some Program 1.0", &HashSet::new()).is_none());
    // And still refuses on the one class no `*` can reach.
    assert!(matches!(
        protection_of(
            &cfg,
            Some("winget"),
            "Some \"Program\" 1.0",
            &HashSet::new()
        ),
        Some(Protection::Undeclarable)
    ));
}
