use super::*;

/// Q36, pinned at the wiring: **adoption reads winget's export, never its listing.**
///
/// `winget list` reports every Add/Remove-Programs and MSIX row with an identifier winget
/// synthesises from the registry — 186 of 280 on the measured host. `winget uninstall`
/// takes those; `winget install` answers `No package found matching input criteria` for
/// every one, and a third of them carry their own version, so the name changes under the
/// declaration when the package updates. Reverting this to `AllInstalled` reads as a
/// simplification and silently replants 186 lines that can never converge.
#[tokio::test]
async fn winget_adoption_reads_what_it_can_reinstall_not_what_it_can_see() {
    let reg = build_registry().await;
    let Some(winget) = reg.get("winget") else {
        return; // not this machine's platform
    };
    let src = winget
        .as_queryable()
        .expect("winget answers questions")
        .manual_source();
    assert!(
        src.contains("export"),
        "winget adoption no longer goes through `winget export`: {src}"
    );
    assert!(
        !src.starts_with("everything "),
        "winget adoption is back on the whole listing, which includes 186 identifiers \
         `winget install` refuses: {src}"
    );
}
