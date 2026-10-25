use super::*;

/// The leading word of a repository command is a program, and for two backends it was a
/// subcommand of a manager that has no such subcommand — `apt add-apt-repository …` and
/// `apk sh -c …`. Both fail on any real host, so `repo add`/`repo remove` had never worked
/// on apt or apk. This is `every_registrar_has_an_argv_row_or_a_written_reason` for
/// the repository surface, and it exists because that test covered install and remove only.
#[tokio::test]
async fn every_repo_row_runs_the_program_that_edits_that_managers_sources() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    type Registrar = fn(&mut BackendRegistry, &CommandExecutor);
    // backend, registrar, the program `repo add` must run, the program `repo list` must run.
    let cases: &[(&str, Registrar, &str, Option<&str>)] = &[
        ("apt", register_apt, "add-apt-repository", None),
        ("apk", register_apk, "sh", Some("cat")),
        ("zypper", register_zypper, "zypper", None),
        ("winget", register_winget, "winget", Some("winget")),
        ("scoop", register_scoop, "scoop", Some("scoop")),
        ("choco", register_choco, "choco", Some("choco")),
        ("gem", register_gem, "gem", Some("gem")),
    ];

    for (name, register, want_write, want_read) in cases {
        let vfs = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let exec =
            CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
        let mut reg = BackendRegistry::new();
        register(&mut reg, &exec);
        let b = reg
            .get(name)
            .unwrap_or_else(|| panic!("{} did not register", name));
        let mgr = b
            .as_repo_manager()
            .unwrap_or_else(|| panic!("{} manages no repositories", name));

        let _ = mgr
            .add_repo("shalltest", "https://example.invalid/repo", false)
            .await;
        if want_read.is_some() {
            let _ = mgr.list_repos().await;
        }
        let calls = mock.get_calls().await;
        assert!(
            calls
                .iter()
                .any(|c| c.split_whitespace().next() == Some(want_write)),
            "{}: repo add ran none of the right program `{}`\n  calls: {:?}",
            name,
            want_write,
            calls
        );
        if let Some(read) = want_read {
            assert!(
                calls
                    .iter()
                    .any(|c| c.split_whitespace().next() == Some(read)),
                "{}: repo list ran none of the right program `{}`\n  calls: {:?}",
                name,
                read,
                calls
            );
        }
    }
}
