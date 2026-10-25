use super::*;

/// A dependency probe sends its operand and keeps the manager's own format language.
///
/// **`{name}` means two things and only one of them is a package.** dnf asks with
/// `--queryformat %{name}`, which is rpm's format language — six characters that a
/// substitute-everywhere fill would have turned into `%jq`, producing a listing of nothing
/// under a command that exits 0. So the operand is the argument that *is* `{name}`, never
/// one that contains it, and that is also what lets it go behind the terminator.
///
/// Pinned for all three because they are the only rows in the tree that ask, and because
/// `argv_drift_tests` does not drive `get_dependencies` — an argv only `shall info` reaches.
#[tokio::test]
async fn a_dependency_probe_sends_the_operand_and_keeps_the_format_string() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    // A backend, the function that builds it, and the argv it must send. Named because
    // spelling a function type inline three deep is what clippy calls complex and a reader
    // calls unreadable.
    type Registrar = fn(&mut BackendRegistry, &CommandExecutor);
    let want: &[(&str, Registrar, &str)] = &[
        (
            "dnf",
            register_dnf as Registrar,
            "dnf repoquery --requires --resolve --queryformat %{name} jq",
        ),
        ("pacman", register_pacman, "pacman -Si -- jq"),
        ("xbps", register_xbps, "xbps-query -x -- jq"),
    ];
    for (name, register, argv) in want {
        let vfs = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let exec =
            CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
        let mut reg = BackendRegistry::new();
        register(&mut reg, &exec);
        let mp = reg
            .get(name)
            .and_then(|b| b.as_metadata_provider().cloned())
            .unwrap_or_else(|| panic!("{name} reports no dependencies"));
        let _ = mp.get_dependencies("jq").await;
        assert_eq!(
            mock.get_calls().await,
            vec![argv.to_string()],
            "{name}'s dependency probe"
        );
    }
}
