use super::*;

#[test]
fn a_man_page_is_not_the_program() {
    let e = entries(&[("fd.1", false), ("fd.bash", false)]);
    assert!(matches!(
        find_executable(&e, "sharkdp/fd", None),
        Err(DiscoveryError::NotFound { .. })
    ));
}
