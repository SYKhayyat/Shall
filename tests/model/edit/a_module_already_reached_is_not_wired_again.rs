use super::*;

#[test]
fn a_module_already_reached_is_not_wired_again() {
    let f = fx(&[
        ("active", "Work\n"),
        ("profiles/Work", "use imperative\n"),
        ("modules/imperative.txt", "apt:curl\n"),
    ]);
    let edit = editor(&f)
        .add(&Landing::Imperative.target(), "apt:jq")
        .unwrap();
    assert_eq!(edit.wired_into, None);
    assert_eq!(
        read(&f, "profiles/Work").matches("use imperative").count(),
        1
    );
}
