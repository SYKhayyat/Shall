use super::*;

#[test]
fn into_a_named_profile_settles_the_question() {
    // Which is why the refusal above points at `--into`: it has to actually work.
    let f = fx(&[
        ("active", "Work\nHome\n"),
        ("profiles/Work", "use dev\n"),
        ("profiles/Home", "use media\n"),
    ]);
    let edit = editor(&f)
        .add(&Target::Profile("Work".into()), "apt:jq")
        .unwrap();
    assert!(read(&f, "profiles/Work").contains("apt:jq"));
    assert_eq!(edit.wired_into, None, "a profile is reached by `active`");
}
