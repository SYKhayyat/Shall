use super::*;

/// The whole state counts when the whole state was surveyed — the control that makes the
/// test above a measurement rather than an artefact of filtering.
#[test]
fn every_manager_that_answered_counts() {
    let state = [
        managed_pkg("brew", "ripgrep"),
        managed_pkg("gem", "rails"),
        managed_pkg("npm", "typescript"),
    ];
    let answered = vec!["brew".to_string(), "gem".to_string(), "npm".to_string()];
    assert_eq!(
        managed_where_the_crawl_could_see(state.iter(), &answered),
        3
    );
}
