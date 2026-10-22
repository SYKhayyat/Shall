use super::*;

/// **`@system` reaches pip as `--break-system-packages`, and only for the line that said
/// it** (`Q49`, owner ruling 2026-08-10).
///
/// The batch split is the half that matters. Without it, one line's permission to write
/// into an OS-owned Python would be handed to every other package in the same wave — which
/// is the exact failure `@unverified` is partitioned to avoid, with a worse blast radius:
/// packages nobody said that about, installed into the system interpreter.
#[tokio::test]
async fn system_reaches_pip_as_its_own_flag_and_is_never_shared() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let inst = GenericInstallable {
        core: Arc::new(pip_core(mock.clone(), vfs)),
    };

    inst.install(&[pip_spec("black", true), pip_spec("httpie", false)], false)
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    let opted: Vec<&String> = calls.iter().filter(|c| c.contains("black")).collect();
    let plain: Vec<&String> = calls.iter().filter(|c| c.contains("httpie")).collect();
    assert!(
        opted.iter().all(|c| c.contains("--break-system-packages")),
        "the line that opted in must carry the flag: {calls:?}"
    );
    assert!(
        plain.iter().all(|c| !c.contains("--break-system-packages")),
        "the line that did NOT opt in must not inherit it: {calls:?}"
    );
    assert!(
        !calls
            .iter()
            .any(|c| c.contains("black") && c.contains("httpie")),
        "the two must not share a command at all: {calls:?}"
    );
}
