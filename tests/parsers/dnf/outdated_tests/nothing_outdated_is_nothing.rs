use super::*;

#[test]
fn nothing_outdated_is_nothing() {
    assert!(parse_dnf_outdated("").is_empty());
    assert!(parse_zypper_outdated("").is_empty());
    // zypper prints its header and nothing beneath when everything is current.
    assert!(parse_zypper_outdated(
        "S  | Repository | Name | Current Version | Available Version | Arch\n"
    )
    .is_empty());
}
