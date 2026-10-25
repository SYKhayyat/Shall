use super::*;

#[test]
fn include_says_to_write_use_instead() {
    // V.46: `use` already means union, and two words for one thing is the disease.
    let f = fx("Work\n", &[("Work", "include editors\n")], &[]);
    let err = resolve(&f).unwrap_err();
    assert!(err.to_string().contains("there is no `include`"), "{}", err);
    assert!(err.hint.unwrap().contains("use editors"));
}
