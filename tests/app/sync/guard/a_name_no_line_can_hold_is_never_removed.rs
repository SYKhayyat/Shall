use super::*;

#[test]
fn a_name_no_line_can_hold_is_never_removed() {
    // Shall must not remove what it could never have been asked to keep: a name that
    // cannot be written down cannot be declared, so it is unmanaged forever and a standing
    // `purge-undeclared` candidate through no fault of its owner.
    //
    // **Y7 moved the boundary, and this test is what says where it is now.** A name with a
    // space in it â€” `ARP\Machine\X64\Android Studio`, which is what `winget list` answers â€”
    // is declarable today, because it can be quoted. What is still beyond a line is a name
    // carrying a quote or a control character, and those are what keep this branch alive.
    let cfg = Config::default();
    let empty = HashSet::new();

    for holdable in [
        r"ARP\Machine\X64\Android Studio",
        "7zip.7zip",
        r"MSIX\Microsoft.BingSearch_1.1.43.0_x64__8wekyb3d8bbwe",
    ] {
        assert!(
            protection_of(&cfg, Some("winget"), holdable, &empty).is_none(),
            "`{holdable}` can be declared, so it is protected by the ordinary rules and \
                 not by undeclarability"
        );
    }

    for unholdable in ["Some \"Quoted\" Program", "two\nlines"] {
        assert!(
            matches!(
                protection_of(&cfg, Some("winget"), unholdable, &empty),
                Some(Protection::Undeclarable)
            ),
            "`{unholdable}` cannot be written as a line and must never be removed"
        );
    }
}
