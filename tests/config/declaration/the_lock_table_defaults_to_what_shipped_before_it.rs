use super::*;

/// **A machine with no `[lock]` section behaves exactly as it did before the table
/// existed.** Asserted key by key rather than by `assert_eq!` on the struct, so a later
/// key added with a bolder default fails here instead of changing every machine quietly.
#[test]
fn the_lock_table_defaults_to_what_shipped_before_it() {
    let lock = LockSettings::default();
    assert!(lock.replay, "an ordinary sync replayed recorded versions");
    assert_eq!(lock.versions, vec!["*".to_string()], "every manager");
    assert_eq!(
        lock.freezes(),
        shall::core::lock_kind::ALL.to_vec(),
        "a bare `lock` froze all nine kinds"
    );
    // And the same reached through `Config`, which is what every caller actually holds —
    // a `Default` on the table that the parent does not use is a default nothing reads.
    assert!(Config::default().lock.replay);
}
