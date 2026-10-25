use super::*;

#[test]
fn referenced_names_finds_every_reference_and_skips_escapes() {
    assert_eq!(referenced_names("when $role == travel"), vec!["role"]);
    assert_eq!(referenced_names("${a}/${b}/$c"), vec!["a", "b", "c"]);
    // `$$` is an escape and `$1` a shell positional — neither is a variable reference.
    assert!(referenced_names("cost $$5 and awk '{print $1}'").is_empty());
    assert!(referenced_names("no refs here").is_empty());
}
