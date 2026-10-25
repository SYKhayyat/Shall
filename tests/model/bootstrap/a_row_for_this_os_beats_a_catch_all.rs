use super::*;

/// A file may carry a general answer and a specific one; the one written for this machine
/// is the one that runs.
#[test]
fn a_row_for_this_os_beats_a_catch_all() {
    let rows = vec![row("brew", None), row("brew", Some("macos"))];
    let picked = for_manager(&rows, "brew", "macos").unwrap();
    assert_eq!(picked.os.as_deref(), Some("macos"));
    // ...and on a machine the specific row does not name, the catch-all still answers.
    assert_eq!(for_manager(&rows, "brew", "linux").unwrap().os, None);
}
