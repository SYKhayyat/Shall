use super::*;

/// **Only what Shall wrote is a candidate** (`#158`).
///
/// The owner's timer kept firing after its `schedules` line was deleted, and the fix is a
/// listing — which is also the shape that can delete something a person registered by hand. The
/// prefix is the whole safety argument: `shall-` for systemd, `com.shall.` for launchd, and a
/// name without one is never returned, however much it looks like a schedule.
#[test]
fn a_unit_directory_yields_only_the_names_shall_wrote() {
    let dir = tempfile::tempdir().unwrap();
    for file in [
        // A whole job, both halves.
        "shall-nightly.service",
        "shall-nightly.timer",
        // A `@reboot` job, service only.
        "shall-boot.service",
        // Not ours: a person's own unit, and a service manager's.
        "my-backup.timer",
        "dbus.service",
        "shall-nightly.conf",
        "Shall_Case.timer",
    ] {
        std::fs::write(dir.path().join(file), "[Unit]\n").unwrap();
    }
    assert_eq!(
        units_in(dir.path()),
        Some(vec!["boot".to_string(), "nightly".to_string()]),
        "a name outside the prefix is a task Shall must never take away, and both halves of one \
         job are one name"
    );
}

/// A directory that is not there holds nothing; one that cannot be read is **not** the same
/// answer, or a converge walks away from units it could not see.
#[test]
fn an_absent_directory_is_empty_and_an_unreadable_one_is_not() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        units_in(&dir.path().join("no-such-dir")),
        Some(Vec::new()),
        "a machine with no unit directory holds no units"
    );
    // A regular file where a directory is expected is an `EIO`-shaped error rather than
    // `NotFound`, which is the cheapest way to make the read fail on every platform.
    let file = dir.path().join("not-a-dir");
    std::fs::write(&file, "x").unwrap();
    assert_eq!(
        units_in(&file),
        None,
        "a directory that could not be read was reported as empty, so the converge would claim \
         this machine holds nothing"
    );
}

/// The launchd twin, and the reason it is a second function rather than a parameter: the label
/// (`com.shall.<name>.plist`) is a different spelling of the same rule.
#[test]
fn a_launch_agents_directory_yields_only_the_labels_shall_wrote() {
    let dir = tempfile::tempdir().unwrap();
    for file in [
        "com.shall.nightly.plist",
        "com.shall.boot.plist",
        "com.example.backup.plist",
        "com.shall.nightly.plist.bak",
    ] {
        std::fs::write(dir.path().join(file), "<plist/>\n").unwrap();
    }
    assert_eq!(
        agents_in(dir.path()),
        Some(vec!["boot".to_string(), "nightly".to_string()])
    );
    assert_eq!(agents_in(&dir.path().join("absent")), Some(Vec::new()));
    assert_eq!(agents_in(&dir.path().join("com.shall.nightly.plist")), None);
}
