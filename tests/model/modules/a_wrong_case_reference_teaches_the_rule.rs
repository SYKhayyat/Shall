use super::*;

#[test]
fn a_wrong_case_reference_teaches_the_rule() {
    // II.5: "no profile named `Editors` — did you mean the module `editors`?"
    let f = fixture(&[("editors.txt", "apt:neovim\n")]);
    let mut loader = ModuleLoader::new(&f.layout, &known);
    let err = loader
        .load("EDITORS_TYPO", &Origin::argument())
        .unwrap_err();
    assert!(err.hint.is_some());
}
