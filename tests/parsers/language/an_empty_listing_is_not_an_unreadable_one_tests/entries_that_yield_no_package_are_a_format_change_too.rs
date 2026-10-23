use super::*;

/// The other half of the same judgement: entries present, none of them readable.
#[test]
fn entries_that_yield_no_package_are_a_format_change_too() {
    // pip capitalising `name`, which is the doc's own example.
    let err = parse_installed("pip", r#"[{"Name":"black","Version":"24.1.0"}]"#)
        .expect_err("an array of entries none of which read is a change");
    assert_eq!(err.backend, "pip");
    assert_eq!(err.data_lines, 1, "the count is of entries, not of lines");

    // npm with the map present and its values shaped differently is NOT this case — the key
    // is the name, so those still read. The case that fails is the map being gone, above.
    assert!(parse_installed("npm", r#"{"dependencies":{"tsc":{}}}"#).is_ok());
}
