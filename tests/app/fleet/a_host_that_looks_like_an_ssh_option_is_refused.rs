use super::*;

#[test]
fn a_host_that_looks_like_an_ssh_option_is_refused() {
    let err = check_host("-oProxyCommand=touch /tmp/pwned").unwrap_err();
    assert!(
        err.to_string().contains("cannot begin with `-`"),
        "the error must say why: {}",
        err
    );
    assert!(check_host("-").is_err());
    check_host("build-01.example.com").unwrap();
    check_host("user@10.0.0.4").unwrap();
}
