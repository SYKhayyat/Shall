use super::*;

#[test]
fn installing_with_several_profiles_active_asks_which_one_rather_than_guessing() {
    // The alternative is picking one of your identities at random, or — worse — writing
    // the line, wiring nothing, and reporting success while installing nothing.
    let f = fx(&[
        ("active", "Work\nHome\n"),
        ("profiles/Work", "use dev\n"),
        ("profiles/Home", "use media\n"),
    ]);
    let err = editor(&f)
        .add(&Landing::Imperative.target(), "apt:jq")
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Work") && msg.contains("Home"), "{}", msg);
    assert!(msg.contains("--into"), "{}", msg);
    // And it wrote nothing: a refusal that half-applies is worse than either answer.
    assert_eq!(read(&f, "modules/imperative.txt"), "");
}
