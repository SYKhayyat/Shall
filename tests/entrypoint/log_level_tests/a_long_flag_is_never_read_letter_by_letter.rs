use super::*;

/// A long flag that merely contains the letters must not be read as one: `--yes` has no
/// `v`, but `--dry-run` and `--verbose-something` are the shapes that catch a naive scan.
#[test]
fn a_long_flag_is_never_read_letter_by_letter() {
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "--dry-run", "--yes", "sync"])),
        "warn"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "--allow-mass-removal", "sync"])),
        "warn"
    );
}
