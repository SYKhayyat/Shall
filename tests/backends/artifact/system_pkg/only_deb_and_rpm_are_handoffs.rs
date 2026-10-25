use super::*;

#[test]
fn only_deb_and_rpm_are_handoffs() {
    assert_eq!(installer_for(Format::Deb), Some("dpkg"));
    assert_eq!(installer_for(Format::Rpm), Some("rpm"));
    // A shape that installs into a database on another OS is not a handoff Shall drives.
    assert_eq!(installer_for(Format::Msi), None);
    assert_eq!(installer_for(Format::Tarball), None);
    assert!(is_handoff_format(Format::Deb));
    assert!(!is_handoff_format(Format::Tarball));
}
