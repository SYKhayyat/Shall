use super::*;

/// The defect this replaced: `--verbose` promised debug logging and delivered none,
/// because the level was read after clap had parsed and the subscriber was already built.
#[test]
fn asking_for_more_gets_more_in_both_spellings() {
    for one in [&["shall", "-v", "list"], &["shall", "--verbose", "list"]] {
        assert_eq!(log_level_from_argv(&argv(one)), "info");
    }
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-vv", "list"])),
        "debug"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-v", "-v", "list"])),
        "debug"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "--verbose", "--verbose", "list"])),
        "debug"
    );
    // Past two there is nothing more to say, and it must not fall back to the default.
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-vvvv", "list"])),
        "debug"
    );
}
