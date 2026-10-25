use super::*;

#[test]
fn an_unknown_when_key_is_an_error_that_lists_the_real_ones() {
    let err = doc("when platform == linux {\n  apt:htop\n}\n")
        .unwrap()
        .statements_for(&facts())
        .unwrap_err();
    assert!(err
        .hint
        .unwrap()
        .contains("os, arch, host, hostname, family, home, user"));
}
