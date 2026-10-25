use super::*;

#[test]
fn a_body_inside_a_when_block_applies_only_there() {
    let body = "when family == debian {\n  github {\n    formats = deb\n  }\n}\n";
    assert_eq!(
        parse(body)
            .unwrap()
            .options("github")
            .unwrap()
            .all("formats"),
        ["deb"]
    );

    let body = "when family == arch {\n  github {\n    formats = deb\n  }\n}\n";
    assert!(parse(body).unwrap().options("github").is_none());
}
