use super::*;

/// **The engine finds the hooks this machine wrote, and fires them.**
///
/// Found by the mutation gate: `SyncEngine::events` replaced with `Default::default()`
/// survived the whole suite. **It is the difference between a hook that runs and a hook that
/// does not** — and a hook is a script this machine asked to run: it pages someone, it opens a
/// ticket, it vetoes a change. The empty default fires nothing, silently, and the user finds
/// out when the notification they expected never arrived.
///
/// **The hooks are read at the moment of firing rather than held**, so a hook edited during a
/// long sync cannot run on an approval given to its previous contents.
#[tokio::test]
async fn the_engine_finds_the_hooks_this_machine_wrote() {
    let dir = config_dir("events");
    // **A hook in the config repo's `hooks/` directory** — the policy every machine should run.
    std::fs::create_dir_all(dir.join("hooks")).expect("the hooks directory");
    std::fs::write(dir.join("hooks").join("on_drift"), "#!/bin/sh\nexit 0\n").expect("the hook");
    // **And one in `preferences.toml`** — where a machine keeps the notification that talks to
    // *its* Slack. **Both run**, repo first: a precedence rule would mean that adding a local
    // notification silently disables the shared policy, which is the quiet failure this model
    // exists to avoid. **The table is `[events]`**, apart from `[hooks]` — that one is the
    // package-lifecycle hooks the embedded interpreter runs, and the two were once the same
    // table, so a `preferences.toml` `after_sync` fired twice.
    let mut config = Config::sandboxed(&dir);
    config.config_root = dir.clone();
    // **The preferences half is in the config the App was handed**, which is where the parsed
    // `[events]` table lives — Shall's own events, apart from the package-lifecycle `[hooks]`
    // the embedded interpreter runs.
    config
        .events
        .insert("on_drift".to_string(), "#!/bin/sh\nexit 0\n".to_string());
    let (engine, _mock) = engine(config).await;

    let hooks = engine.events();
    assert!(
        !hooks.is_empty(),
        "the engine found no hooks at all, so nothing this machine wrote would ever run"
    );
    assert_eq!(
        hooks.all().len(),
        2,
        "both locations run — the repo's policy and this machine's own: {:?}",
        hooks.all()
    );
    assert!(
        hooks
            .all()
            .iter()
            .all(|h| h.event == shall::model::event::Event::OnDrift),
        "the hooks are the ones for the event that was written: {:?}",
        hooks.all()
    );
}
