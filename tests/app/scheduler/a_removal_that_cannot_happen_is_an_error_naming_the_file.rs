use super::*;

#[test]
fn a_removal_that_cannot_happen_is_an_error_naming_the_file() {
    // The point is that the failure is reported at all: swallowing it left a timer armed
    // under a schedule Shall had just reported as removed. Making a path undeletable is
    // the platform-specific part; the assertion is not.
    //
    // It used to be a directory, back when the removal was `remove_file` and a directory
    // was therefore undeletable by it. `force_remove` deletes directories on purpose, so
    // that stand-in silently became a success — a test asserting an error over a call that
    // could no longer produce one.
    let dir = tempfile::tempdir().unwrap();
    let unit = dir.path().join("shall-nightly.timer");
    std::fs::write(&unit, b"[Timer]\n").unwrap();

    // Windows: an open handle with no sharing at all. `File::open` will not do — Rust's
    // default share mode includes `FILE_SHARE_DELETE`, so a plain open leaves the file
    // perfectly deletable, which is how the first attempt at this test passed nothing.
    #[cfg(windows)]
    let _held = {
        use std::os::windows::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&unit)
            .unwrap()
    };
    // Unix: a parent nobody may write. Root ignores the mode, so the check below can be
    // vacuous in a container running as root — Windows carries this one in CI.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    }

    let outcome = remove_generated(&unit);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        if outcome.is_ok() {
            return; // running as root; the path was deletable after all
        }
    }
    let err = outcome.unwrap_err().to_string();
    assert!(
        err.contains("shall-nightly.timer"),
        "the error does not name the file: {}",
        err
    );
    assert!(
        err.contains("still be armed"),
        "the error does not say what is left behind: {}",
        err
    );
}
