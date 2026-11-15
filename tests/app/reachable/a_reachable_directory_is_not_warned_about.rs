use super::*;
use std::ffi::OsString;

/// A directory that IS on PATH produces no warning — otherwise every install on a
/// correctly configured machine nags.
///
/// **The predicate is asked about a given `PATH`, not about the process's.** Rewriting the
/// process's `PATH` was what made this test flake the whole binary: every test thread spawns
/// through that one string, so a directory that happens to be on it — the fake manager
/// `tool_help` runs — disappears wherever this test has briefly replaced it. The value is
/// supplied here, so the true and false arms are both exact and the process is left alone.
#[test]
fn a_reachable_directory_is_not_warned_about() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("bin");
    std::fs::create_dir_all(&dir).unwrap();

    let absent = if cfg!(windows) {
        OsString::from("C:\\shall-test-not-on-path")
    } else {
        OsString::from("/shall-test-not-on-path")
    };
    assert!(
        !is_on_path_at(&absent, &dir),
        "a directory named by no entry was read as on PATH"
    );

    let mut with_entry = OsString::from(dir.as_os_str());
    if cfg!(windows) {
        with_entry.push(";C:\\Windows");
    } else {
        with_entry.push(":/usr/bin");
    }
    assert!(
        is_on_path_at(&with_entry, &dir),
        "a directory that IS on PATH was reported as missing"
    );

    // A symlink spelling the same directory is one directory, not two: `canonicalize`, not
    // string equality, is the comparison that keeps `~/go/bin` and `/home/u/go/bin` apart
    // from a path nobody would write while still matching a `PATH` entry that links here.
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link = tmp.path().join("bin-through-a-link");
        symlink(&dir, &link).unwrap();
        let through_link = OsString::from(link.as_os_str());
        assert!(
            is_on_path_at(&through_link, &dir),
            "a PATH entry reached through a symlink was not read as this directory"
        );
    }
}
