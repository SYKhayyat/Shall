use super::*;

#[test]
fn a_zypper_header_row_is_not_a_package() {
    assert!(!parse_zypper_outdated(ZYPPER)
        .iter()
        .any(|p| p.name == "Name"));
}
