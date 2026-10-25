use super::*;

#[test]
fn a_scoped_option_is_refused_on_a_backend_that_cannot_read_it() {
    for b in ["apt", "cargo", "npm", "github", "brew"] {
        for (key, _, _) in SCOPED_OPTIONS {
            assert!(!takes_scoped_option(b, key), "{} takes @{}", b, key);
        }
    }
    // The neighbours inside the family are the ones worth naming: each storage backend
    // takes a different subset, and a table that let them share everything would put
    // `@size` on a subvolume that has no size to set.
    assert!(!takes_scoped_option("zfs", "size"));
    assert!(!takes_scoped_option("btrfs", "size"));
    assert!(!takes_scoped_option("zfs", "mount_options"));
    assert!(!takes_scoped_option("lvm", "quota"));
}
