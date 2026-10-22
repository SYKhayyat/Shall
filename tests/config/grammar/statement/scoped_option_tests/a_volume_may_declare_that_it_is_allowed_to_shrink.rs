use super::*;

/// Q19's opt-out. A declared `@size` is applied to a volume that already exists, and the one
/// direction that can destroy a filesystem is written on the line rather than assumed.
#[test]
fn a_volume_may_declare_that_it_is_allowed_to_shrink() {
    assert_eq!(
        opt("lvm:vg0/data@size=5G,allow_shrink=true", "allow_shrink"),
        "true"
    );
    assert_eq!(
        opt("lvm:vg0/data@size=5G,allow_shrink", "allow_shrink"),
        "true"
    );

    // Nowhere else. A quota is a limit, not a filesystem, so lowering one destroys nothing
    // and there is nothing here to permit — the flag on those backends would read as a
    // safety measure that guards nothing.
    for line in [
        "zfs:tank/data@quota=5G,allow_shrink=true",
        "btrfs:/mnt/fs/data@quota=5G,allow_shrink=true",
        "apt:curl@allow_shrink=true",
    ] {
        let err = format!("{}", p(line).unwrap_err());
        assert!(
            err.contains("`@allow_shrink` is not an option on"),
            "`{}`: {}",
            line,
            err
        );
    }
}
