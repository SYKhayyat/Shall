use super::*;

#[test]
fn all_is_a_mode_and_not_a_filename() {
    let p = AssetPattern::parse("all").unwrap();
    assert!(p.installs_every_match());
    assert!(p.matches("literally-anything"));
}
