use super::*;

/// **A drive-relative name is an escape, and `is_absolute()` calls it false.**
///
/// `C:evil` resolves against the current directory on drive C. `join` drops the bin
/// directory for it exactly as it does for `C:\evil`, so the confinement that only asked
/// `is_absolute()` handed the write wherever the process happened to be.
#[cfg(windows)]
#[test]
fn a_drive_relative_bin_name_is_refused() {
    for bad in [
        "C:evil",
        "x:evil",
        "C:Windows",
        r"C:\evil",
        r"\\host\share\evil",
    ] {
        let out = bin_destination(&dir(), bad, true);
        assert!(out.is_err(), "`{}` must be refused, got {:?}", bad, out);
    }
}
