use super::*;

#[test]
fn bare_atom_resolves_to_itself() {
    assert_eq!(eval_str("apt:jq", &[]).unwrap(), vec!["apt:jq"]);
}
