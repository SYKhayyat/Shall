use super::*;

/// The row must name the path, or the test above passes over a table that says nothing.
#[test]
fn the_systemd_row_names_the_file_sd_booted_checks() {
    assert_eq!(
        shipped("systemd").detect_file.as_deref(),
        Some("/run/systemd/system"),
        "sd_booted(3) checks this path; a different one here is a guess"
    );
    assert!(
        shipped("sysvinit").detect_file.is_none(),
        "SysVinit has no daemon to be running — its command IS the whole test"
    );
}
