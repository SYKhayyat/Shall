use super::*;

/// The not-found case, which is where junk rows come from in three of the four managers
/// `names_only` still serves.
#[test]
fn pixi_finding_nothing_yields_nothing() {
    assert!(pixi_search(NOT_FOUND, "pixi").is_empty());
    assert!(pixi_search("", "pixi").is_empty());
}
