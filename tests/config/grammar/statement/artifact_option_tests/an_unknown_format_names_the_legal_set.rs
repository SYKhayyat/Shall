use super::*;

#[test]
fn an_unknown_format_names_the_legal_set() {
    let err = p("github:sharkdp/fd@formats=snapcraft").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("snapcraft"));
    assert!(
        msg.contains("appimage"),
        "the error must list the vocabulary"
    );
}
