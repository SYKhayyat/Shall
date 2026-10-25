use super::*;

#[test]
fn a_deb_is_installable_only_where_dpkg_exists() {
    // D5: on a machine with the manager the file is a handoff install; without it, the
    // format is not installable here, so a line offering only a `.deb` becomes download-only
    // rather than failing.
    assert!(installable_here(Format::Deb, true, false));
    assert!(!installable_here(Format::Deb, false, false));
    assert!(installable_here(Format::Rpm, false, true));
    assert!(!installable_here(Format::Rpm, false, false));
    // Archives never depend on a system installer.
    assert!(installable_here(Format::Tarball, false, false));
    assert!(installable_here(Format::Binary, false, false));
    // A macOS/Windows database shape Shall does not hand off is not installable via this path.
    assert!(!installable_here(Format::Msi, true, true));
}
