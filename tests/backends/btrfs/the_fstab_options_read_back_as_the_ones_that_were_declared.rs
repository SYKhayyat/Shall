use super::*;

/// The inverse of what `fstab_with` writes, so a changed `@mount_options` can be seen. The
/// `subvol=` is dropped because it is the address, not an option the user declared — leaving
/// it in would make every entry compare unequal to the options that produced it.
#[test]
fn the_fstab_options_read_back_as_the_ones_that_were_declared() {
    for declared in ["defaults", "noatime", "noatime,compress=zstd"] {
        let written = fstab_with("", "abc", "/data", "/srv", declared);
        assert_eq!(
            fstab_options(&written, "/data").as_deref(),
            Some(declared),
            "round trip through the entry changed {}",
            declared
        );
    }
    // A subvolume with no entry has no options, which is not the same as `defaults`: the
    // planner must be able to tell "not written" from "written with the default".
    assert_eq!(fstab_options("", "/data"), None);
    assert_eq!(
        fstab_options(
            &fstab_with("", "abc", "/data", "/srv", "defaults"),
            "/other"
        ),
        None
    );
}
