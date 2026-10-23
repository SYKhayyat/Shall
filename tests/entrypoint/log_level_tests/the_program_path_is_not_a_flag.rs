use super::*;

/// argv[0] is a path, and on this developer's machine it contains a `v` (`Videos`) and on
/// plenty of others a `q`. It is never a flag.
#[test]
fn the_program_path_is_not_a_flag() {
    assert_eq!(
        log_level_from_argv(&argv(&["/home/q/Videos/shall", "list"])),
        "warn"
    );
}
