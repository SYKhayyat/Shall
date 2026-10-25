use super::*;

#[test]
fn a_declaration_with_no_scope_still_names_its_file() {
    // A line in a profile belongs to no module, and an imperative spec to neither.
    let d = Declaration {
        at: "profiles/Work:5".into(),
        scopes: vec![],
        lapsed: false,
        from_regex: None,
    };
    assert_eq!(d.describe(), "at profiles/Work:5");
}
