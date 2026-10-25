use super::*;

/// The `[hooks]` table belongs to the package-lifecycle hooks; event hooks read `[events]`.
/// When both read `[hooks]`, a `preferences.toml` `after_sync` fired twice — once as Lua,
/// once as a script — so an `after_sync` under `[hooks]` must not be seen here at all.
#[test]
fn an_after_sync_under_hooks_is_not_an_event_hook() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = config_at(tmp.path());
    config.hooks.insert(
        "after_sync".into(),
        [("*".to_string(), "print('lua')".to_string())]
            .into_iter()
            .collect(),
    );
    assert!(
        EventHooks::load(&config).is_empty(),
        "an [events] hook must not be read out of the [hooks] table"
    );
}
