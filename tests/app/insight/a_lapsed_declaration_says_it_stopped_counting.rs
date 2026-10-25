use super::*;

#[test]
fn a_lapsed_declaration_says_it_stopped_counting() {
    // II.16: an expired line lingers in your file. `why` must not report it as the
    // reason a package is present when it has stopped being that reason.
    let d = Declaration {
        at: "modules/imperative.txt:2".into(),
        scopes: vec!["module:imperative".into()],
        lapsed: true,
        from_regex: None,
    };
    assert!(d.describe().contains("expired, so it no longer counts"));
}
