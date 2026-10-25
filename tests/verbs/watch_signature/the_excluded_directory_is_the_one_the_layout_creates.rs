use super::*;

/// **`locks` is `Layout`'s name, not this module's.** Two copies of one name drift, and the
/// drift is a daemon that wakes itself up.
#[test]
fn the_excluded_directory_is_the_one_the_layout_creates() {
    let layout = shall::model::Layout::new(
        "/tmp/shall-watch-locks-check",
        "/tmp/shall-watch-locks-data",
    );
    let locks = layout
        .locks_dir()
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .expect("the locks dir has a name");
    assert!(
        NEVER_WATCHED.contains(&locks.as_str()),
        "the layout writes `locks/` and this module excludes {:?}",
        NEVER_WATCHED
    );
}
