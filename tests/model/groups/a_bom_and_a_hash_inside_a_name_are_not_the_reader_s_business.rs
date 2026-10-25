use super::*;

/// The hygiene the modules reader already had: a byte-order mark at the start of the file
/// is invisible, and a `#` only opens a comment where whitespace precedes it — so a member
/// whose NAME carries a hash survives intact. A BOM'd groups file used to fail with a
/// refusal about the wrong thing, blaming the priority list.
#[test]
fn a_bom_and_a_hash_inside_a_name_are_not_the_reader_s_business() {
    let text = "\u{feff}tools = apt, dnf, car#go\n";
    let g = Groups::parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        g.expand("tools"),
        Some(vec!["apt".to_string(), "dnf".to_string(), "car#go".to_string()].as_slice())
    );
}
