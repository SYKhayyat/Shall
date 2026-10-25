use super::*;

/// `winget`'s own identifiers, and the four things widening the allowlist must NOT do.
///
/// Found by running the native sweep: the grammar was taught to accept a backslash in a
/// name (G-2) and this validator was not, so `adopt` wrote 340 winget rows it believed it
/// could write and the next command could not parse the file — `adopted.txt:78`, a wedged
/// model, which is E1's class arriving through the other door.
#[test]
fn winget_identifiers_are_names_and_the_widening_stops_there() {
    for name in [
        r"ARP\Machine\X64\{8BD2A40D-67A6-45F5-877D-6D9D04C9D5A2}",
        r"ARP\Machine\X86\ILST_30_2_1",
        r"MSIX\Microsoft.AV1VideoExtension_2.0.24.0_x64__8wekyb3d8bbwe",
        "7zip.7zip",
        // The space, which is 6 of the 161 names this box could not declare.
        r"ARP\Machine\X64\Mozilla Firefox",
    ] {
        assert!(
            Validator::validate_package_name_for(name, "winget").is_ok(),
            "winget prints `{name}` and cannot be handed it back"
        );
    }

    // 0. The space is legal *inside* a name and never at its edge. A trailing space is
    //    invisible in a manifest, in a listing and in every message about either, so
    //    `Firefox ` and `Firefox` would be one package that reconciles as two — and a name
    //    made only of spaces names nothing while looking like a name. The grammar refuses
    //    the same shapes; a rule kept on one side of a round trip is a rule the other side
    //    breaks.
    for edged in [
        r"ARP\Machine\X64\Mozilla Firefox ",
        r" ARP\Machine\X64\Mozilla Firefox",
        " ",
        "   ",
    ] {
        assert!(
            Validator::validate_package_name_for(edged, "winget").is_err(),
            "`{edged}` was accepted, so an invisible edge is a second package"
        );
    }

    // 1. Only winget. Every other backend keeps the strict allowlist.
    assert!(
        Validator::validate_package_name_for(r"ARP\Machine\X64\thing", "cargo").is_err(),
        "the widening leaked to a backend whose manager never prints such a name"
    );
    // 2. Traversal is still forbidden, for winget as for everything else.
    assert!(
        Validator::validate_package_name_for(r"ARP\..\..\Windows\System32", "winget").is_err(),
        "`..` must stay refused whatever else the name may carry"
    );
    // 3. The shell metacharacters that are NOT part of a Windows identifier stay blocked.
    for hostile in [
        r"ARP\Machine; rm -rf /",
        r"ARP\Machine`whoami`",
        r"ARP\Machine$(id)",
        r"ARP\Machine|cat",
    ] {
        assert!(
            Validator::validate_package_name_for(hostile, "winget").is_err(),
            "`{hostile}` is not an identifier, it is a command line"
        );
    }
    // 4. And the ordinary names every other backend depends on still pass.
    for (name, backend) in [
        ("@angular/cli", "npm"),
        ("serde_json", "cargo"),
        ("sharkdp/fd", "github"),
    ] {
        assert!(
            Validator::validate_package_name_for(name, backend).is_ok(),
            "`{name}` stopped being a legal {backend} name"
        );
    }
}
