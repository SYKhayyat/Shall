use super::*;

/// Every package carries where it came from: the point of this output is answering "why is
/// this here", and an answer with no origin cannot be checked.
#[test]
fn every_package_names_the_line_that_declared_it() {
    let mut state = shall::model::DesiredState::default();
    state
        .packages
        .insert("apt".to_string(), vec![spec("apt", "jq", true)]);
    let doc = Evaluation::of(&state, Path::new("/repo"));
    assert!(
        doc.present[0]
            .source
            .as_deref()
            .is_some_and(|s| s.contains("modules/x.txt")),
        "{:?}",
        doc.present[0]
    );
}
