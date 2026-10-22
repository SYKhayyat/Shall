use super::*;

#[test]
fn channel_on_a_backend_without_version_streams_is_an_error() {
    let err = p("github:sharkdp/fd@channel=stable").unwrap_err();
    assert!(format!("{}", err).contains("not an option on `github`"));
}
