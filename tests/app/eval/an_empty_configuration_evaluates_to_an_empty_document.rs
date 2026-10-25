use super::*;

#[test]
fn an_empty_configuration_evaluates_to_an_empty_document() {
    let doc = Evaluation::of(&shall::model::DesiredState::default(), Path::new("/repo"));
    assert!(doc.present.is_empty() && doc.absent.is_empty() && doc.extras.is_empty());
    // ...and still serialises, so a consumer never has to special-case "nothing".
    assert!(serde_json::to_string(&doc).is_ok());
}
