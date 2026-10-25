use super::*;

#[test]
fn the_when_arm_that_matched_beats_the_plain_line_below_it() {
    // Same first-mention-wins rule the order uses, so the two cannot disagree.
    let body = "when family == debian {\n  github {\n    formats = deb\n  }\n}\n\
                github {\n  formats = binary\n}\n";
    assert_eq!(
        parse(body)
            .unwrap()
            .options("github")
            .unwrap()
            .all("formats"),
        ["deb"]
    );
}
