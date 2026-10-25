use super::*;

#[test]
fn a_gnu_style_manager_terminates_and_a_windows_one_does_not() {
    assert!(terminates_options("apt-get"));
    assert!(terminates_options("pacman"));
    assert!(terminates_options("brew"));
    // `scoop` is still the unasked one of the three Windows managers: a PowerShell script
    // dispatching on `$args[0]`. `winget` and `choco` were listed beside it on the same
    // reasoning and the probe disproved both, which is the reason this row is the one left.
    assert!(!terminates_options("scoop"));
    assert!(terminates_options("winget"));
    assert!(terminates_options("choco"));
}
