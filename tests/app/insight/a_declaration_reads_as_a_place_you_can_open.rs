use super::*;

#[test]
fn a_declaration_reads_as_a_place_you_can_open() {
    // `why` answers "where is this declared?", so the answer is a file and a line — not
    // a label like "manifest: local.txt" that names a file the model no longer reads.
    let d = Declaration {
        at: "modules/dev.txt:3".into(),
        scopes: vec!["module:dev".into(), "profile:Work".into()],
        lapsed: false,
        from_regex: None,
    };
    assert_eq!(
        d.describe(),
        "at modules/dev.txt:3 (module:dev, profile:Work)"
    );
}
