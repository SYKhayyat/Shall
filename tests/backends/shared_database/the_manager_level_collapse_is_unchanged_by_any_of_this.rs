use super::*;

/// The manager-level collapse has no package in hand — `check health` refreshes managers,
/// not packages — so it must keep answering with the owner and never consult a set it was
/// not given.
#[test]
fn the_manager_level_collapse_is_unchanged_by_any_of_this() {
    let mut all = vec!["pacman".to_string(), "yay".into(), "paru".into()];
    one_backend_per_shared_database(&mut all);
    assert_eq!(all, vec!["pacman"]);
}
