use super::*;

#[test]
fn installing_writes_a_line_to_the_landing_module() {
    let f = fx(&[("active", "Work\n"), ("profiles/Work", "use dev\n")]);
    let edit = editor(&f)
        .add(&Landing::Imperative.target(), "apt:jq")
        .unwrap();
    assert!(read(&f, "modules/imperative.txt").contains("apt:jq"));
    assert_eq!(edit.line, "apt:jq");
}
