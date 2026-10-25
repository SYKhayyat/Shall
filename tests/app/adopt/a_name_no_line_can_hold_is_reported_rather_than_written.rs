use super::*;

#[test]
fn a_name_no_line_can_hold_is_reported_rather_than_written() {
    // `winget list` answers for Add/Remove-Programs entries with pseudo-IDs like
    // `ARP\Machine\X64\Android Studio`. Written into `modules/adopted.txt`, that is a
    // parse error in a file Shall generated — and since every later command parses the
    // model, one such name wedged the whole config until it was hand-edited. Found by
    // the live Windows sweep, where `rollback` died on adopted.txt:69.

    // **Y7 moved the boundary.** A name with a space is written quoted now, so the ARP row
    // that once wedged the config is adopted rather than held back. What is still beyond a
    // line is a name carrying a quote — one line cannot express it, and this is the case
    // that keeps the hold-back alive.
    // A resource is adopted like a package now (ruled 2026-08-03): the whole family, so a
    // `link:` or `setting:` cannot quietly keep the old behaviour after `service:` lost it.
    let options = HashMap::from([(
        "service".to_string(),
        vec![("status".to_string(), "running".to_string())],
    )]);
    let mut found = Discovery {
        adopt: vec![
            Package::new("7zip.7zip", "winget"),
            Package::new(r"ARP\Machine\X64\Android Studio", "winget"),
            Package::new("Some \"Quoted\" Program", "winget"),
            Package::new("AppMgmt", "service"),
            Package::new("/home/u/.vimrc", "link"),
        ],
        ..Default::default()
    };
    Adopter::hold_back_what_cannot_be_written(&mut found, &options);

    assert_eq!(found.adopt.len(), 4, "every writable name is adopted");
    for adopted in [
        "7zip.7zip",
        r"ARP\Machine\X64\Android Studio",
        "AppMgmt",
        "/home/u/.vimrc",
    ] {
        assert!(
            found.adopt.iter().any(|p| p.name == adopted),
            "`{adopted}` can be written, so it is a line and not a comment"
        );
    }
    assert_eq!(
        found.skipped.len(),
        1,
        "and what could not be taken is reported, not dropped"
    );
    assert!(
        found.skipped[0].reason.contains("no line can hold"),
        "the manifest has to name what it could not take: {}",
        found.skipped[0].reason
    );
}
