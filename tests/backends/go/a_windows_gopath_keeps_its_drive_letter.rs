use super::*;

/// GOPATH is a list in the platform's own separator. Splitting on `:` as well as `;`
/// decapitated every Windows path at the drive letter, so the bin dir resolved to `C\bin`,
/// which does not exist — and a missing bin dir is reported as "nothing installed".
///
/// Windows only, and that is the point rather than a gap: on Unix a colon *is* the
/// separator, so the same string is two entries there and reading it as one would be the
/// bug. `the_first_entry_of_a_gopath_list_owns_bin` pins that half, on both platforms.
#[cfg(windows)]
#[test]
fn a_windows_gopath_keeps_its_drive_letter() {
    let bin = gopath_bin(GOPATH_WINDOWS).unwrap();
    assert_eq!(bin, PathBuf::from(r"C:\Users\Administrator\go").join("bin"));
}
