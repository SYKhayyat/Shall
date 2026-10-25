use super::*;

#[test]
fn a_row_for_another_os_is_not_this_machines_store() {
    let mut elsewhere = row("elsewhere", "cmd");
    elsewhere.os = Some("plan9".into());
    assert!(!elsewhere.applies_here());
    let mut here = row("here", "cmd");
    here.os = Some(std::env::consts::OS.to_string());
    assert!(here.applies_here());
}
