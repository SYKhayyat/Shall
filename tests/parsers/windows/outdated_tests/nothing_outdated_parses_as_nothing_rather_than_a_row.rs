use super::*;

#[test]
fn nothing_outdated_parses_as_nothing_rather_than_a_row() {
    assert!(parse_winget_outdated("").is_empty());
    assert!(parse_scoop_outdated("").is_empty());
    assert!(parse_choco_outdated("").is_empty());
    // Header with no rows beneath it.
    assert!(parse_scoop_outdated(
        "Name    Installed Version Latest Version Missing Dependencies Info\n----    ----------------- -------------- -------------------- ----\n"
    )
    .is_empty());
}
