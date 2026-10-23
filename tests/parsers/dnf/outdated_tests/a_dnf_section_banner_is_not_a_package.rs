use super::*;

/// `Upgrades` is a section banner, not a package.
#[test]
fn a_dnf_section_banner_is_not_a_package() {
    assert!(!parse_dnf_outdated(DNF).iter().any(|p| p.name == "Upgrades"));
}
