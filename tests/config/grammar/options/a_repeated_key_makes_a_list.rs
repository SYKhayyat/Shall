use super::*;

#[test]
fn a_repeated_key_makes_a_list() {
    let mut opts = Options::default();
    opts.insert("requires".to_string(), "apt:libfoo");
    opts.insert("requires".to_string(), "apt:libbar");
    assert_eq!(opts.all("requires"), ["apt:libfoo", "apt:libbar"]);
}
