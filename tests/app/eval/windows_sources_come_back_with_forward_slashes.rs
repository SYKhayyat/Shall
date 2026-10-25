use super::*;

/// The other half of the same contract, and it can only be asserted here: a backslash is
/// a separator on Windows and an ordinary filename character everywhere else, so
/// `Path::strip_prefix` splits `C:\repo\x` into components on Windows and treats it as one
/// name on unix. Asserting it unconditionally is a test that passes on the machine it was
/// written on and fails on both others — which is what it did.
#[cfg(windows)]
#[test]
fn windows_sources_come_back_with_forward_slashes() {
    assert_eq!(
        repo_relative(r"C:\repo\modules\x.txt:7", Path::new(r"C:\repo")),
        "modules/x.txt:7"
    );
    assert_eq!(
        repo_relative(r"C:\repo\active", Path::new(r"C:\repo")),
        "active"
    );
}
