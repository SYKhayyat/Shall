use super::*;

/// Q20's argv. Both switches ride one refresh: a snap needing a channel *and* a confinement
/// change used to get only the channel, because the branch that built the refresh looked at
/// `@channel` alone.
#[test]
fn a_refresh_carries_every_switch_the_declaration_asks_for() {
    let ch = "stable".to_string();
    assert_eq!(
        refresh_args("code", Some(&ch), false).unwrap(),
        vec!["refresh", "--channel", "stable", "--", "code"]
    );
    assert_eq!(
        refresh_args("code", None, true).unwrap(),
        vec!["refresh", "--classic", "--", "code"]
    );
    assert_eq!(
        refresh_args("code", Some(&ch), true).unwrap(),
        vec!["refresh", "--channel", "stable", "--classic", "--", "code"],
        "a line carrying both options must not lose one"
    );
    // Nothing to change is not a refresh — the caller decides what to do instead.
    assert!(refresh_args("code", None, false).is_none());
}
