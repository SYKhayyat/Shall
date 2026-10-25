use super::*;

/// Adding a repository, removing one and listing them are three commands, and a row may
/// name a different program for each.
///
/// **The argv table drives `install` and `remove` and nothing else**, so a repository verb
/// has never been in it — which is how apt spent months running `apt add-apt-repository`,
/// a command apt refuses (`S44`). Converting `dnf` and `pacman` to rows put two more
/// three-program managers on that path: dnf adds with its own plugin and removes a file
/// with `rm`, and pacman writes both through `sh` and reads through `pacman-conf`. Each
/// combination is asserted here, because "it compiled" says nothing about which program ran.
#[tokio::test]
async fn a_repository_verb_runs_the_program_that_verb_needs() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    type Registrar = fn(&mut BackendRegistry, &CommandExecutor);
    for (name, register) in [
        ("dnf", register_dnf as Registrar),
        ("pacman", register_pacman),
    ] {
        let vfs = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let exec =
            CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
        let mut reg = BackendRegistry::new();
        register(&mut reg, &exec);
        let rm = reg
            .get(name)
            .and_then(|b| b.as_repo_manager().cloned())
            .unwrap_or_else(|| panic!("{name} manages no repositories"));

        rm.add_repo("shallprobe", "https://example.invalid/r", false)
            .await
            .unwrap_or_else(|e| panic!("{name} add_repo: {e}"));
        rm.remove_repo("shallprobe", false, removal_token().await)
            .await
            .unwrap_or_else(|e| panic!("{name} remove_repo: {e}"));
        let _ = rm.list_repos().await;

        let calls = mock.get_calls().await;
        let want: &[&str] = match name {
            "dnf" => &[
                "dnf config-manager --add-repo https://example.invalid/r",
                "rm -f /etc/yum.repos.d/shallprobe.repo",
                "dnf repolist --all",
            ],
            _ => &[
                "sh -c set -e; printf",
                "sh -c rm -f '/etc/pacman.d/shall-shallprobe.conf'",
                "pacman-conf --repo-list",
            ],
        };
        for w in want {
            assert!(
                calls.iter().any(|c| c.contains(w)),
                "{name}: no call contained `{w}` — it ran {calls:?}"
            );
        }
    }

    // The name lands in a path, so it is validated as a path segment — which is what both
    // hand-written modules did and the shared repo path did not, because until these two
    // became rows no row put a name in a path.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    register_dnf(&mut reg, &exec);
    let rm = reg.get("dnf").unwrap().as_repo_manager().unwrap().clone();
    for escape in ["../../etc/cron.d/x", "..", "", "a/b"] {
        assert!(
            rm.remove_repo(escape, false, removal_token().await)
                .await
                .is_err(),
            "`{escape}` became part of `/etc/yum.repos.d/<name>.repo` unchallenged"
        );
    }
    assert!(
        mock.get_calls().await.is_empty(),
        "a refused repository name still ran something: {:?}",
        mock.get_calls().await
    );
    // The control: a real name still works, so this is not a check that refuses everything.
    rm.remove_repo("epel", false, removal_token().await)
        .await
        .expect("a plain name");
}
