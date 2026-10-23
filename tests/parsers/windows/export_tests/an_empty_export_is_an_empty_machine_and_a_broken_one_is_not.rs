use super::*;

/// An export that did not happen must not read as a machine with nothing on it. The file's
/// absence is an error the caller raises; a file that exists and says nothing is winget
/// saying there is nothing to restore, which is a different and legitimate answer.
#[test]
fn an_empty_export_is_an_empty_machine_and_a_broken_one_is_not() {
    // Five inputs, and this test used to assert one answer for all of them. Two of them are
    // winget saying it has nothing to export, which is a fact. The other three are winget
    // failing, and reading those as "nothing installed" is what `adopt` then writes down and
    // `sync` then acts on.
    for empty in [
        r#"{"Sources": []}"#,
        r#"{"Sources": [{"SourceDetails": {}}]}"#,
    ] {
        assert!(
            parse_winget_export(empty)
                .expect("a source list with no packages is an export of nothing")
                .is_empty(),
            "{empty}"
        );
    }
    for broken in ["", "not json at all", r#"{"WinGetVersion": "1.12.360"}"#] {
        let err = parse_winget_export(broken)
            .expect_err("an export that could not be read is not an empty machine");
        assert_eq!(err.backend, "winget", "{broken}");
    }
}
