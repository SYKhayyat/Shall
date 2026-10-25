use super::*;

/// Rendered output ends in a newline — a JSON document without one puts the next shell
/// prompt in the middle of it.
#[test]
fn the_rendered_document_ends_in_a_newline() {
    let out = Evaluation::of(&shall::model::DesiredState::default(), Path::new("/repo"))
        .render()
        .unwrap();
    assert!(out.ends_with("}\n"), "{:?}", out);
}
