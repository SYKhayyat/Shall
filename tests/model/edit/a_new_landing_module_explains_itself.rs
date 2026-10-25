use super::*;

#[test]
fn a_new_landing_module_explains_itself() {
    // V.40: provenance ends up in the filename, and the file says what it is for.
    let f = fx(&[("active", "Work\n"), ("profiles/Work", "\n")]);
    editor(&f)
        .add(&Landing::Hooks.target(), "apt:htop")
        .unwrap();
    let body = read(&f, "modules/hooks.txt");
    assert!(body.contains("behind Shall's back"), "{}", body);
    assert!(body.contains("apt:htop"));
}
