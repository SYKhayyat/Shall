use super::*;

/// A malicious traversal path never even reaches the placement list.
#[test]
fn a_traversal_path_is_dropped_from_the_plan() {
    let files = vec![PathBuf::from("modules/../../../etc/cron.d/evil")];
    assert!(plan(&files, &|_| false).placements.is_empty());
}
