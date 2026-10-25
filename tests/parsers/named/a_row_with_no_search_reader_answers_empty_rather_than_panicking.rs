use super::*;

/// A row with no `searches` has no `Searchable` either, so this vector is unreachable
/// through a capability — asserted here so the claim is checked rather than believed.
#[test]
fn a_row_with_no_search_reader_answers_empty_rather_than_panicking() {
    let p = NamedParser::new("helm", installed("ws_name_version").unwrap(), None, None);
    assert!(p.parse_search("anything").is_empty());
}
