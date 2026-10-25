use super::*;

#[test]
fn a_missing_profile_that_matches_a_module_teaches_the_rule() {
    // II.5's exact message.
    let f = fixture(&[], &[("editors.txt", "apt:neovim\n")]);
    let err = resolve(&f, "Editors").unwrap_err();
    assert!(err.what.contains("no profile named `Editors`"), "{}", err);
    let hint = err.hint.unwrap();
    assert!(
        hint.contains("did you mean the module `editors`"),
        "{}",
        hint
    );
    assert!(hint.contains("Profiles are Capitalized, modules are lowercase"));
}
