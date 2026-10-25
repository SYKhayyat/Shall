use super::*;

#[cfg(windows)]
#[test]
fn windows_forbidden_paths_match_case_insensitively_and_whole_components_only() {
    // NTFS compares names without case, so a differently-cased path meets the same wall.
    assert!(
        Validator::refuse_forbidden(Path::new(r"\\?\c:\windows\SYSTEM32\config\sam",)).is_err()
    );
    // A prefix match is on whole components: nothing under the sun named SAM* is caught
    // by the SAM entry.
    assert!(
        Validator::refuse_forbidden(Path::new(r"\\?\C:\Windows\System32\config\SAMARITAN",))
            .is_ok()
    );
}
