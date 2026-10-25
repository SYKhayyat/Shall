use super::*;

/// `shall clean-cache` reaches the managers that have one, and says nothing about the rest.
///
/// **Three outcomes and no fourth**, the same discipline the argv table applies to install
/// and remove — because the fourth, *"it reported success having run nothing"*, is exactly
/// what forty backends did. `GenericUpgradable` had no `clean_cache` at all: every manager
/// on the data path answered `Unsupported`, `handle_clean_cache` filtered that out
/// silently, and a Debian machine with a full `/var/cache/apt/archives` was told **"No
/// backend on this machine has a cache to clear."**
///
/// The argv is pinned for the four system managers because a typo in a verb only
/// `clean-cache` reaches is invisible on every platform that cannot run the manager — the
/// whole reason the argv table exists.
#[tokio::test]
async fn cache_cleaning_runs_a_command_or_refuses_by_name() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    type Registrar = fn(&mut BackendRegistry, &CommandExecutor);
    // The rows whose argv is worth pinning by name: the system managers, whose cache is the
    // one that fills a disk, and the one that empties it with a different binary.
    let pinned: &[(&str, Registrar, &str)] = &[
        ("apt", register_apt, "apt-get clean"),
        ("dnf", register_dnf, "dnf clean all"),
        ("pacman", register_pacman, "pacman -Sc --noconfirm"),
        ("xbps", register_xbps, "xbps-remove -Oy"),
        ("zypper", register_zypper, "zypper clean --all"),
    ];
    for (name, register, want) in pinned {
        let vfs = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let exec =
            CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
        let mut reg = BackendRegistry::new();
        register(&mut reg, &exec);
        let up = reg
            .get(name)
            .and_then(|b| b.as_upgradable().cloned())
            .unwrap_or_else(|| panic!("{name} cannot upgrade"));
        up.clean_cache(false)
            .await
            .unwrap_or_else(|e| panic!("{name} has a cache verb and refused it: {e}"));
        let calls = mock.get_calls().await;
        assert!(
            calls.iter().any(|c| c.contains(want)),
            "{name}: cache cleaning ran no call containing `{want}` — it ran {calls:?}"
        );
    }

    // And the whole family: no backend may report success without running anything, and
    // none may run something and then report it has no cache.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    for case in argv_cases() {
        (case.register)(&mut reg, &exec);
    }
    let mut silent_success: Vec<String> = Vec::new();
    let mut ran_then_refused: Vec<String> = Vec::new();
    let mut cleaners: Vec<String> = Vec::new();
    for b in reg.all() {
        let Some(up) = b.as_upgradable() else {
            continue;
        };
        let before = mock.get_calls().await.len();
        let outcome = up.clean_cache(false).await;
        let after = mock.get_calls().await.len();
        let name = b.name().to_string();
        match (outcome, after > before) {
            (Ok(()), true) => cleaners.push(name),
            (Ok(()), false) => silent_success.push(name),
            (Err(shall::core::Error::Unsupported(_)), false) => {}
            (Err(_), true) => ran_then_refused.push(name),
            // A manager that ran its verb and the mock failed it, or refused for a reason
            // other than "no such verb" — both are the caller's to report, not this gate's.
            (Err(_), false) => {}
        }
    }
    assert!(
        silent_success.is_empty(),
        "these backends reported a cleared cache without running anything: {silent_success:?}"
    );
    assert!(
        ran_then_refused.is_empty(),
        "these backends ran a cache command and then reported failure: {ran_then_refused:?}"
    );
    assert!(
        cleaners.len() >= 15,
        "only {} backend(s) can clear a cache — the data path had none of them until \
         `clean_cache` became a row, and a low number here is that returning: {cleaners:?}",
        cleaners.len()
    );
}
