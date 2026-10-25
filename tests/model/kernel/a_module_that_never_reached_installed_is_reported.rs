use super::*;

/// A module still at `added` or `built` after autoinstall has run is one that did not make
/// it — the fact worth failing on.
#[test]
fn a_module_that_never_reached_installed_is_reported() {
    let out = "\
nvidia/550.78, 6.8.0-35-generic, x86_64: built
v4l2loopback/0.12.7, 6.8.0-35-generic, x86_64: installed
";
    let stuck = not_installed(out);
    assert_eq!(stuck.len(), 1, "{:?}", stuck);
    assert_eq!(stuck[0].name, "nvidia");
}
