use super::*;

/// A user's row comes first, so disagreeing with a shipped one is writing a row rather
/// than editing Shall — and a row for another OS is not offered at all.
#[test]
fn rows_are_selected_by_manager_and_os() {
    let mut theirs = row("mix", &["mine"]);
    theirs.os = Some("linux".into());
    let mut elsewhere = row("mix", &["theirs"]);
    elsewhere.os = Some("macos".into());
    let rows = vec![theirs, elsewhere, row("mix", &["shipped"])];
    let picked = for_manager(&rows, "mix", "linux");
    assert_eq!(picked.len(), 2, "the macos row is not for this machine");
    assert_eq!(picked[0].run, vec!["mine".to_string()]);
    assert!(for_manager(&rows, "opam", "linux").is_empty());
}
