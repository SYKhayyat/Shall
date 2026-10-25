use super::*;

#[tokio::test]
async fn every_backend_sends_the_argv_its_manager_expects() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    let mut unterminated: Vec<String> = Vec::new();
    let mut split_locks: Vec<String> = Vec::new();
    for case in argv_cases() {
        let vfs = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let locks = Arc::new(DashMap::new());
        let exec = CommandExecutor::with_layer(true, false, mock.clone(), vfs, locks.clone());
        let mut reg = BackendRegistry::new();
        (case.register)(&mut reg, &exec);

        let name = case.backend;
        let b = reg
            .get(name)
            .unwrap_or_else(|| panic!("{name} did not register"));
        let inst = b
            .as_installable()
            .unwrap_or_else(|| panic!("{name} cannot install"));
        let spec = shall::core::PackageSpec {
            name: case.subject.into(),
            backend: name.into(),
            options: case
                .options
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            ..Default::default()
        };

        let installed = inst.install(&[spec], false).await;
        let after_install = mock.get_calls().await.len();
        // The removal's existence answers go in here, between the two drives, so an
        // install's ask and a removal's ask of the same command can each get the phase's
        // own truth.
        for (cmd, out) in case.remove_stubs {
            mock.set_response(
                cmd,
                Ok(shall::core::executor::DryRunOutput {
                    stdout: out.as_bytes().to_vec(),
                    stderr: vec![],
                }
                .into()),
            );
        }
        let removed = inst
            .remove(&[case.subject.to_string()], false, removal_token().await)
            .await;
        let calls = mock.get_calls().await;

        check(
            name,
            "install",
            &case.install,
            installed,
            &calls[..after_install],
        );
        check(
            name,
            "remove",
            &case.remove,
            removed,
            &calls[after_install..],
        );
        unterminated.extend(operands_outside_the_terminator(&case, &calls));

        // One manager, one lock. The map is keyed by whatever each call asked to be
        // exclusive over, so more than one key here is a backend whose install and whose
        // removal do not wait for each other.
        let mut keys: Vec<String> = locks.iter().map(|e| e.key().clone()).collect();
        keys.sort();
        if keys.len() > 1 {
            split_locks.push(format!("{}: {:?}", case.backend, keys));
        }
    }

    assert!(
        split_locks.is_empty(),
        "these backends take a DIFFERENT exclusive lock to install than to remove, so two \
         Shall processes can write one package database at the same time:\n    {}\n\n\
         The lock names the manager, not the program: a manager that installs with one \
         binary and removes with another has one database and two names for it.",
        split_locks.join("\n    ")
    );

    assert!(
        unterminated.is_empty(),
        "these invocations hand a declaration's own text to a program that ends its options \
         at `--`, without sending one:\n    {}\n\n\
         `core::argv::push_names` is what puts the terminator in, and it is where the \
         answer for each program lives. A backend that builds argv by hand is a backend \
         that has to remember, and the two that did are the two root-privileged system \
         managers.",
        unterminated.join("\n    ")
    );
}
