use super::*;

#[test]
fn a_pin_no_tag_answers_names_both_spellings_it_tried() {
    let err = one_release("sharkdp/fd", "9.9.9", None, None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("9.9.9"), "{}", err);
    assert!(err.contains("v9.9.9"), "{}", err);
}
