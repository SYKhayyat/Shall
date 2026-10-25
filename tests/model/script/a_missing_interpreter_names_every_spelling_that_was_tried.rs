use super::*;

#[test]
fn a_missing_interpreter_names_every_spelling_that_was_tried() {
    let err = launch_for(
        Path::new("/tmp/hook"),
        "#!/usr/bin/env definitely-not-installed-anywhere\n",
    )
    .expect_err("no such interpreter");
    let text = err.to_string();
    assert!(
        text.contains("definitely-not-installed-anywhere"),
        "{}",
        text
    );
    assert!(text.contains("PATH"), "{}", text);
}
