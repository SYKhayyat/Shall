use super::*;

#[test]
fn the_rhel_derivatives_share_one_family() {
    for id in ["centos", "rocky", "almalinux"] {
        assert_eq!(
            parse_os_release_family(&format!("ID={}\n", id)).as_deref(),
            Some("rhel"),
            "{} should be rhel",
            id
        );
    }
}
