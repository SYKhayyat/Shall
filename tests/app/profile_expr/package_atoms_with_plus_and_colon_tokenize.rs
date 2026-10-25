use super::*;

#[test]
fn package_atoms_with_plus_and_colon_tokenize() {
    let r = eval_str("apt:g++ | cargo:ripgrep", &[]).unwrap();
    assert_eq!(r, vec!["apt:g++", "cargo:ripgrep"]);
}
