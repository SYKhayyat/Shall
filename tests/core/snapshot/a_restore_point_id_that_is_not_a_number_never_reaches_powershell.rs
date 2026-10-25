use super::*;

#[test]
fn a_restore_point_id_that_is_not_a_number_never_reaches_powershell() {
    // SEC5. Both PowerShell strings interpolate the id unquoted and run elevated.
    assert_eq!(windows_sequence_number("42").unwrap(), 42);
    assert_eq!(windows_sequence_number("  7 ").unwrap(), 7);
    for bad in [
        "1); Start-Process calc; #",
        "-1",
        "1 2",
        "",
        "$(whoami)",
        "0x10",
    ] {
        assert!(
            windows_sequence_number(bad).is_err(),
            "`{}` must be refused",
            bad
        );
    }
}
