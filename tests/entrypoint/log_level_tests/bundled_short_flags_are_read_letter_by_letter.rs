use super::*;

/// A short flag can arrive bundled with its neighbours, and every letter in the bundle
/// counts — `-nv` is a dry run that talks.
#[test]
fn bundled_short_flags_are_read_letter_by_letter() {
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-nv", "sync"])),
        "info"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-nvv", "sync"])),
        "debug"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-nq", "sync"])),
        "error"
    );
}
