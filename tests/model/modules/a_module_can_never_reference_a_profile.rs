use super::*;

#[test]
fn a_module_can_never_reference_a_profile() {
    // The layering rule (II.3, V.2). Otherwise "what does `editors` contain?" depends
    // on what you activated.
    let f = fixture(&[("editors.txt", "use Work\n")]);
    let err = expand_module(&f, "editors").unwrap_err();
    assert!(err.what.contains("cannot `use` the profile"), "{}", err);
    assert!(err.hint.unwrap().contains("profiles choose; modules hold"));
}
