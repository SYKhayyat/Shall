use super::*;

/// The same rule one level in. `@mount_options` fills the fstab entry `@mount` writes, so
/// without `@mount` it reaches nothing — which is the line-that-does-nothing class this
/// whole ruling is about, and it would have shipped inside the fix for it.
#[test]
fn mount_options_without_a_mount_is_refused() {
    let err = format!(
        "{}",
        p("btrfs:/mnt/fs/data@mount_options=noatime").unwrap_err()
    );
    assert!(err.contains("`@mount_options` has no `@mount`"), "{}", err);
    // With the mount it is exactly what it says.
    assert_eq!(
        opt(
            "btrfs:/mnt/fs/data@mount=/srv,mount_options=noatime",
            "mount_options"
        ),
        "noatime"
    );
    // And a bare `@mount` needs no options — `defaults` is what the entry gets.
    assert_eq!(opt("btrfs:/mnt/fs/data@mount=/srv", "mount"), "/srv");
}
