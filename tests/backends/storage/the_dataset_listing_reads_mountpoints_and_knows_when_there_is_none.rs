use super::*;

/// The listing carries where each dataset is mounted, so `sync` can tell a `@mount=` that
/// took effect from one that did not. `legacy`, `none` and `-` are ZFS saying it mounts this
/// nowhere, and none of them is a path — reporting one as if it were would leave a declared
/// mountpoint looking satisfied by a word.
#[test]
fn the_dataset_listing_reads_mountpoints_and_knows_when_there_is_none() {
    let pkgs = parse_zfs_list(
        "tank\t0\t/tank\n\
         tank/data\t0\t/mnt/data\n\
         tank/legacy\t0\tlegacy\n\
         tank/hidden\t0\tnone\n\
         tank/blank\t0\t-\n\
         tank/spaced\t0\t/mnt/my data\n\
         \n",
    );
    let at = |n: &str| {
        pkgs.iter()
            .find(|p| p.name == n)
            .unwrap_or_else(|| panic!("{} was not listed", n))
            .properties
            .get("mount")
            .map(String::as_str)
    };
    assert_eq!(pkgs.len(), 6, "a blank line is not a dataset");
    assert_eq!(at("tank/data"), Some("/mnt/data"));
    assert_eq!(
        at("tank/spaced"),
        Some("/mnt/my data"),
        "tab-separated, not whitespace"
    );
    // Three spellings of "ZFS is not mounting this". None is a path, so none is reported as
    // one — a declared `@mount=` against any of them is unsatisfied, which is what the
    // planner needs to see.
    assert_eq!(at("tank/legacy"), None);
    assert_eq!(at("tank/hidden"), None);
    assert_eq!(at("tank/blank"), None);
}
