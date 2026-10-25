use super::*;

/// The shipped rows go through the loader, not around it. If this fails, the built-in
/// adapters have stopped being the thing a user's row is.
#[test]
fn the_shipped_table_parses_and_carries_gsettings() {
    let all = adapters(vec![]);
    let names: Vec<&str> = all.iter().map(|a| a.name.as_str()).collect();
    // The shipped rows are unfiltered here (OS selection happens in `adapter()`), so both
    // built-ins are present regardless of host.
    assert!(names.contains(&"gsettings"), "{:?}", names);
    assert!(names.contains(&"windows-registry"), "{:?}", names);
    let gs = all.iter().find(|a| a.name == "gsettings").unwrap();
    assert_eq!(gs.detect, "gsettings");
    assert_eq!(gs.os.as_deref(), Some("linux"));
}
