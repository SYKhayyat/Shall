use super::*;

/// A row that could not tell whether it is needed would act on every sync.
#[test]
fn a_row_that_cannot_answer_is_refused() {
    let mut r = row("mix", &["mix", "local.hex"]);
    r.probe = vec!["".into()];
    assert!(r.unusable().is_some());
    r.probe = vec!["mix".into(), "hex.info".into()];
    r.run = vec![];
    assert!(r.unusable().is_some());
}
