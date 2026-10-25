use super::*;

/// Every OS-native backend's install and remove argv, checked on whatever host runs the
/// suite.
///
/// These registrars were `#[cfg(target_os = …)]` until 2026-07-26, so `mas`'s verbs were
/// only ever compiled on a Mac and `apt`'s only on Linux — a typo in either was invisible
/// to every other platform's CI, and there is no Mac in this project at all. They are
/// compiled everywhere now and still *registered* only on their own OS, which is the part
/// that has to stay true: `create_default_registry` keeps its `cfg!` gate, and
/// `registry_capability_matrix` asserts what this host actually offers.
/// A system package manager resolves its own dependency closure, so Shall must not
/// re-derive one: `expand_transitive_dependencies` turns every returned name into an
/// install node, and a name that is not a package is then installed by name.
///
/// `zypper` was the only system manager that asked, and it is the one whose first real run
/// could not install anything — `zypper info --requires jq` answered with `Loading`,
/// `Reading`, `No` and twenty other words it had printed, and three of them required each
/// other in a cycle. This asserts the whole family agrees, not just the one that broke.
#[tokio::test]
async fn no_self_resolving_system_manager_re_derives_a_dependency_closure() {
    type Registrar = fn(&mut BackendRegistry, &CommandExecutor);
    // `mut` is used by the BSD-target pushes below; other targets never touch it again.
    #[allow(unused_mut)]
    let mut system: Vec<(&str, Registrar)> = vec![
        ("apt", register_apt),
        ("apk", register_apk),
        ("zypper", register_zypper),
        ("winget", register_winget),
        ("scoop", register_scoop),
        ("choco", register_choco),
        ("guix", register_guix),
        ("emerge", register_emerge),
        ("eopkg", register_eopkg),
        ("slackpkg", register_slackpkg),
        ("yay", register_yay),
        ("paru", register_paru),
    ];
    // Same gating as the registration site: each is compiled where its packaging
    // universe lives.
    #[cfg(target_os = "netbsd")]
    system.push(("pkgin", register_pkgin));
    #[cfg(target_os = "freebsd")]
    system.push(("pkg", register_pkg_freebsd));
    #[cfg(target_os = "openbsd")]
    system.push(("pkg_add", register_pkg_add_openbsd));

    let mut asks: Vec<String> = Vec::new();
    for (name, register) in system {
        let vfs = Arc::new(dashmap::DashMap::new());
        let mock_calls = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
        let exec = CommandExecutor::with_layer(
            true,
            false,
            mock_calls.clone(),
            vfs,
            Arc::new(dashmap::DashMap::new()),
        );
        let mut reg = BackendRegistry::new();
        register(&mut reg, &exec);
        let b = reg
            .get(name)
            .unwrap_or_else(|| panic!("{} did not register", name));
        let Some(mp) = b.as_metadata_provider() else {
            continue;
        };
        let _ = mp.get_dependencies("jq").await;
        // The assertion is that it RAN NOTHING. Checking the returned `Vec` instead would
        // pass on every manager whether or not it asked, because an unmocked command
        // answers with nothing — the vacuous check this repo keeps rediscovering.
        let ran = mock_calls.get_calls().await;
        if !ran.is_empty() {
            asks.push(format!("{name} ran {ran:?}"));
        }
    }
    assert!(
        asks.is_empty(),
        "these system managers re-derive a dependency closure their own installer already \
         resolves, and every name they return becomes an install node: {asks:?}"
    );
}
