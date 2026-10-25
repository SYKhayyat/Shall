use super::*;

/// Approving one adapter file must not approve its siblings. They carry different argv and
/// an edit to one is not a review of the other.
#[test]
fn each_adapter_file_is_approved_on_its_own() {
    let tmp = tempfile::tempdir().unwrap();
    let adapters = tmp.path().join("adapters");
    std::fs::create_dir_all(&adapters).unwrap();
    std::fs::write(adapters.join("backends.toml"), BACKENDS).unwrap();
    std::fs::write(adapters.join("settings.toml"), SETTINGS).unwrap();
    let locks = tmp.path().join("locks");

    // Approve only the backends file.
    approve(&locks, "backends.toml", BACKENDS);

    assert!(
        read_approved_definitions(&adapters.join("backends.toml"), &locks).is_some(),
        "the approved file did not load"
    );
    assert!(
        read_approved_definitions(&adapters.join("settings.toml"), &locks).is_none(),
        "approving backends.toml also approved settings.toml"
    );

    // Approve the sibling too, and now both load.
    approve(&locks, "settings.toml", SETTINGS);
    assert!(read_approved_definitions(&adapters.join("settings.toml"), &locks).is_some());
}
