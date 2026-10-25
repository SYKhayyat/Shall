use super::*;

#[test]
fn the_first_write_wires_the_module_into_the_active_profile_and_says_so() {
    // II.8: a normal line you can read and delete. Never implicit.
    let f = fx(&[("active", "Work\n"), ("profiles/Work", "use dev\n")]);
    let edit = editor(&f)
        .add(&Landing::Imperative.target(), "apt:jq")
        .unwrap();
    assert_eq!(edit.wired_into.as_deref(), Some("Work"));
    assert!(read(&f, "profiles/Work").contains("use imperative"));
    assert!(edit.describe("Added").contains("Work"));
}
