use super::*;

/// **A removal that left the name answering owes the user a sentence** (`#136`, C11).
///
/// `nimble uninstall -y nimjson` returns 0 and drops the package from its own listing, and
/// `nimjson.cmd` stays in `~/.nimble/bin`, which is on PATH — measured on the Windows sweep
/// (`42361477256`, job `128360605190`). Shall deletes no file a manager wrote, so the honest
/// ending is to name it, and **the engine is where every removal passes** (LX-5): `sync`, `apply
/// --delete`, `execute`, `remove-orphans`, `purge-undeclared` and `rollback` all arrive here, so
/// a sentence written in one verb is a sentence five verbs do not say.
///
/// `appimage` is the backend the sandbox can aim: its executables are deployed into `[bin_dir]`,
/// which is the test's own directory, so the file the manager "left" is one this test wrote and
/// the reader is the same one the product uses.
#[tokio::test]
async fn a_removal_names_the_launcher_the_manager_left() {
    let sandbox = config_dir("leftover-launcher");
    let config = Config::sandboxed(&sandbox);
    let bin = config.bin_dir.clone();
    std::fs::create_dir_all(&bin).expect("the deploy directory");
    let launcher = bin.join("nimjson.cmd");
    std::fs::write(&launcher, b"@echo off\n").expect("the leftover launcher");

    let (engine, _mock) = engine(config).await;

    let (logs, guard) = log_collector();
    engine
        .sync(
            removal_changes(&[("appimage", "nimjson")]),
            GuardScope::Sync,
        )
        .await
        .expect("the removal failed");
    drop(guard);

    let logs = logs.lock().expect("the log mutex").clone();
    assert!(
        logs.iter()
            .any(|line| line.contains("left") && line.contains("nimjson.cmd")),
        "a removal that left the name answering said nothing about it: {logs:?}"
    );

    // **And an install-only run owes no sentence about the same file.** The warning is about a
    // removal, not about a directory that happens to hold something with the package's name —
    // and a `SyncEngine` that warned on every run would be a paragraph nobody reads.
    let (logs, guard) = log_collector();
    engine
        .sync(
            install_changes(&[package("brew", "neovim")]),
            GuardScope::Sync,
        )
        .await
        .expect("the install failed");
    drop(guard);

    let logs = logs.lock().expect("the log mutex").clone();
    assert!(
        !logs.iter().any(|line| line.contains("nimjson.cmd")),
        "an install-only run accused something of leaving a launcher: {logs:?}"
    );
}
