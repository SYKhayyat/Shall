use super::*;

#[test]
fn a_package_a_pattern_matched_names_the_pattern() {
    // II.15: no line says `fonts-cantarell`. Sending the reader to `modules/dev.txt:3`
    // without saying why would send them to a line that does not mention the package.
    let d = Declaration {
        at: "modules/dev.txt:3".into(),
        scopes: vec!["module:dev".into()],
        lapsed: false,
        from_regex: Some("^fonts-".into()),
    };
    assert_eq!(
        d.describe(),
        "matched by `re:^fonts-` at modules/dev.txt:3 (module:dev)"
    );
}
