use super::*;

/// The built-in defs parse, and the invariants the shipped rows must hold (U27, V.82): every
/// row is usable, apfs/btrfs are create-only (V.60), windows is a typed-PowerShell row, and
/// zfs registers last so btrfs wins on a machine with both (U28).
#[test]
fn the_builtin_snapshot_defs_hold_their_invariants() {
    let file: SnapshotProviderFile = toml::from_str(BUILTIN_SNAPSHOT_DEFS).unwrap();
    let by = |name: &str| {
        file.snapshot
            .iter()
            .find(|d| d.name == name)
            .unwrap()
            .clone()
    };

    for d in &file.snapshot {
        // detect_path/source placeholders are filled at load, so is_usable (which does not
        // check them) must pass for every shipped row as written.
        assert!(d.unusable().is_none(), "{} must be a usable row", d.name);
    }

    // Create-only, on purpose — claiming Live would be the V.60 lie.
    assert!(!by("apfs").is_live(), "apfs is create-only");
    assert!(
        !by("btrfs").is_live(),
        "btrfs cannot restore a running root"
    );
    // Live restore.
    assert!(by("zfs").is_live());
    assert!(by("timeshift").is_live());
    assert!(by("windows_restore").is_live());

    // Windows is the one typed-PowerShell row (V.82); nothing else is a shell line.
    assert!(by("windows_restore").powershell);
    for name in ["btrfs", "zfs", "timeshift", "apfs"] {
        assert!(!by(name).powershell, "{} must be plain argv", name);
    }

    // zfs ships last so btrfs wins on a machine with both (U28 registration order).
    let names: Vec<&str> = file.snapshot.iter().map(|d| d.name.as_str()).collect();
    let btrfs_at = names.iter().position(|n| *n == "btrfs").unwrap();
    let zfs_at = names.iter().position(|n| *n == "zfs").unwrap();
    assert!(btrfs_at < zfs_at, "btrfs must register before zfs");
}
