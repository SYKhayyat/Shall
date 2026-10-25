use super::*;

/// Only the three real events are read. A file named after a typo must not be picked up
/// as some other event's hook.
#[test]
fn a_file_named_after_a_typo_is_not_loaded() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("hooks");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("after-sync"), "#!/bin/sh\ntrue\n").unwrap();
    std::fs::write(dir.join("on_drifts"), "#!/bin/sh\ntrue\n").unwrap();
    assert!(EventHooks::load(&config_at(tmp.path())).is_empty());
}
