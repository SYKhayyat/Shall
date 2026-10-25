use super::*;

/// `btrfs:` names a subvolume by its filesystem path — `btrfs subvolume create <path>` is
/// the whole install — and it was absent from the path-oriented list until 2026-07-30. So
/// the one backend whose name is *literally* a filesystem path was the one the list forgot,
/// and no declaration of it could be written:
///
/// ```text
/// $ shall -y install btrfs:/mnt/data/vol
/// Error: Validation error: Path traversal detected in name: /mnt/data/vol
/// ```
///
/// Found by the first privileged container run in the project's history, because until
/// there was a real btrfs filesystem to install into, nothing ever tried.
///
/// The family is every backend whose name may begin with a separator. `lvm:vg/lv`,
/// `zfs:pool/dataset` and `setting:SCHEMA/KEY` all carry a separator and never a leading
/// one, so the strict rule is right for them and they are asserted here to keep it that way.
#[test]
fn a_backend_whose_name_is_a_path_may_say_so_and_the_others_may_not() {
    for good in [
        "/mnt/data/vol",
        "/mnt/shall-btrfs/canary",
        "/.snapshots/root",
    ] {
        assert!(
            Validator::validate_package_name_for(good, "btrfs").is_ok(),
            "`btrfs:{good}` is a subvolume path and the install runs `subvolume create` on it"
        );
    }
    // The bans that make widening the list narrow: traversal, injection, and the allowlist.
    assert!(Validator::validate_package_name_for("/mnt/../etc/shadow", "btrfs").is_err());
    assert!(Validator::validate_package_name_for("/mnt/$(id)", "btrfs").is_err());

    // The siblings that must NOT be widened — each names a path-shaped thing that is not a
    // filesystem path, so a leading separator is still an injection attempt.
    for (name, backend) in [
        ("/vg0/data", "lvm"),
        ("/tank/data", "zfs"),
        ("/org.gnome.desktop/idle-delay", "setting"),
    ] {
        assert!(
            Validator::validate_package_name_for(name, backend).is_err(),
            "`{backend}:` names {name} with no leading separator; allowing one would widen \
             the guard for nothing"
        );
    }
}
