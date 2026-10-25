use super::*;

#[test]
fn installing_with_nothing_active_refuses_rather_than_writing_a_line_nobody_reads() {
    let f = fx(&[("active", "\n")]);
    let err = editor(&f)
        .add(&Landing::Imperative.target(), "apt:jq")
        .unwrap_err();
    assert!(err.to_string().contains("nothing is active"), "{}", err);
    assert_eq!(read(&f, "modules/imperative.txt"), "");
}
