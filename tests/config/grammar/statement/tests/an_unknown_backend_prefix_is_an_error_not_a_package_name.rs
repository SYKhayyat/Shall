use super::*;

#[test]
fn an_unknown_backend_prefix_is_an_error_not_a_package_name() {
    // C13. Six of the eight old parsers did `split_once(':')` and trusted the prefix,
    // so a typo became a backend and every new prefix was read as one.
    let err = p("aptt:curl").unwrap_err();
    assert!(err.what.contains("not a backend"), "{}", err.what);
    assert!(err.hint.unwrap().contains("priority"));
}
