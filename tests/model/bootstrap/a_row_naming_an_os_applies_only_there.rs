use super::*;

#[test]
fn a_row_naming_an_os_applies_only_there() {
    let r = row("brew", Some("macos"));
    assert!(r.applies_to("macos"));
    assert!(!r.applies_to("linux"));
}
