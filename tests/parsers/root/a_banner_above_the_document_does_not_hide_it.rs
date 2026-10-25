use super::*;

/// composer prints `Changed current directory to /root/.composer` ahead of every global
/// command, and a `--json` reader that starts at byte zero reads that as a syntax error and
/// answers "nothing installed" about a full machine.
#[test]
fn a_banner_above_the_document_does_not_hide_it() {
    let composer = COMPOSER_WITH_BANNER;
    let doc = json_document(composer).expect("the document below the banner");
    assert_eq!(doc["installed"][0]["name"], "psr/log");

    // The same output read from byte zero, which is what every one of these readers did.
    assert!(
        serde_json::from_str::<serde_json::Value>(composer).is_err(),
        "if this ever parses, the fixture stopped reproducing the bug"
    );
}
