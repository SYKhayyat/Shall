use super::*;

#[test]
fn an_unknown_format_is_refused_here_too() {
    let err = ArtifactOptions::read(&opts(&[("formats", "deb;nonsense")])).unwrap_err();
    assert!(err.contains("nonsense"));
}
