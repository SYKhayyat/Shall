use super::*;

/// The version has to survive the fix, or the cure removes the answer with the junk.
#[test]
fn choco_search_keeps_the_version() {
    let found = parse_choco_search(CHOCO_MACHINE);
    let rg = found.iter().find(|p| p.name == "ripgrep").expect("ripgrep");
    assert_eq!(rg.version.as_deref(), Some("14.1.0"));
}
