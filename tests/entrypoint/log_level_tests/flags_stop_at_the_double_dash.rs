use super::*;

/// Everything after `--` is the command's, not Shall's. A script named `-v` does not
/// turn logging on, and `shall run -- mytool -q` does not silence Shall.
#[test]
fn flags_stop_at_the_double_dash() {
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "run", "--", "mytool", "-v"])),
        "warn"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "run", "--", "mytool", "-q"])),
        "warn"
    );
}
