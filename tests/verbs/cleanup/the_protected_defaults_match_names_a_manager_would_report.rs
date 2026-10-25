use super::*;

/// The platform lists name packages the managers actually report.
///
/// They used to read `kernel32`, `ntdll.dll`, `win32`, `xnu` — the OS's vocabulary, not any
/// manager's, so nothing ever matched them and outside Linux the protected set was three
/// names. A default list that matches nothing is indistinguishable from no list, and this
/// is what tells the two apart.
#[test]
fn the_protected_defaults_match_names_a_manager_would_report() {
    let cfg = Config::default();
    let protected = |name: &str| cfg.protection_rule(name).is_some();

    // Shared, on every platform.
    assert!(protected("sudo") && protected("bash") && protected("shall"));

    #[cfg(target_os = "windows")]
    {
        // Exactly as `winget list` and `choco list` print them.
        assert!(protected("Microsoft.VCRedist.2015+.x64"));
        assert!(protected("vcredist140"));
        assert!(protected("Microsoft.DotNet.DesktopRuntime.8"));
        assert!(protected("Git.Git"));
        // And a name nobody should be stopped from managing.
        assert!(!protected("GitHub.cli"));
        assert!(!protected("dotPDN.PaintDotNet"));
    }
    #[cfg(target_os = "macos")]
    {
        assert!(protected("ca-certificates"));
        assert!(protected("openssl@3"), "the family, under its versions");
        // brew's git and curl are a preference, not a dependency: macOS ships its own in
        // /usr/bin, so protecting them would be friction bought with no safety.
        assert!(!protected("git") && !protected("curl"));
    }
}
