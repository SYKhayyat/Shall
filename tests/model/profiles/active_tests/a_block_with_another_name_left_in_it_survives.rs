use super::*;

#[test]
fn a_block_with_another_name_left_in_it_survives() {
    let e = deactivate(
        "when host == laptop {\n  Travel\n  Work\n}\n",
        "laptop",
        &["Travel"],
    );
    assert!(e.emptied.is_empty());
    assert_eq!(e.body, "when host == laptop {\n  Work\n}\n");
}
