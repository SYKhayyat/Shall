use super::*;

/// An empty file is not a hook. Touching `hooks/on_drift` to remind yourself to write one
/// must not make Shall execute nothing and call it a success — nor refuse the sync because
/// nothing is unapproved.
#[test]
fn an_empty_hook_file_is_not_a_hook() {
    let tmp = tempfile::tempdir().unwrap();
    write_repo_hook(tmp.path(), Event::OnDrift, "   \n\n");
    assert!(EventHooks::load(&config_at(tmp.path())).is_empty());
}
