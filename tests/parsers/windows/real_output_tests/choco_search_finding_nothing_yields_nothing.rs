use super::*;

/// The empty case. `names_only`'s only test used a spack fixture and said nothing about
/// the four other managers routed through it; this is the same trap one file over.
#[test]
fn choco_search_finding_nothing_yields_nothing() {
    assert!(parse_choco_search(CHOCO_NOT_FOUND).is_empty());
    assert!(parse_choco_search("").is_empty());
}
