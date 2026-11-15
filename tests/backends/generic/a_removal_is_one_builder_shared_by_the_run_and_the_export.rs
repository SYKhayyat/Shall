use super::*;

/// **The exporter and `remove` have to split a batch at the same points (#67, II.97).**
///
/// `planned_removals` exists so a bootstrap script can carry an `absent:` line. The risk it
/// creates is the one `planned_installs` was written to avoid: a second description of "how apt
/// removes a package" that is free to disagree with the first. Here the risk is sharper than for
/// installs, because the agreement has to hold across the operand caps too — a script that puts
/// 150 names on one `apt remove` line fails on the machine where Shall, splitting at 100, would
/// have worked.
///
/// So the assertion is not "a command was returned": it is that the commands the exporter is
/// handed are byte-identical to the ones a sync would build, chunk for chunk.
fn apt_core_with_removal_args(remove_args: &[&str]) -> GenericBackendCore {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let mut core = apt_like_core(mock, vfs);
    core.config.remove_args = remove_args.iter().map(|s| s.to_string()).collect();
    core
}

fn specs(names: &[String]) -> Vec<PackageSpec> {
    names
        .iter()
        .map(|n| PackageSpec {
            name: n.clone(),
            backend: "apt".into(),
            ..Default::default()
        })
        .collect()
}

#[test]
fn every_declared_removal_becomes_a_command() {
    let core = Arc::new(apt_core_with_removal_args(&["remove", "-y"]));
    let installable = GenericInstallable { core };

    let commands = installable
        .planned_removals(&specs(&["snap".to_string(), "firefox".to_string()]))
        .expect("apt has a removal verb, so it can say which command removes a package");

    assert_eq!(commands.len(), 1, "two short names fit in one command");
    assert_eq!(commands[0].binary, "apt");
    assert_eq!(
        commands[0].args,
        vec!["remove", "-y", "--", "snap", "firefox"],
        "the argv is not the one `remove` builds"
    );
}

/// **The split, at both caps, against the same builder the run uses.** This is the half a test
/// asserting "some commands came back" would miss entirely.
#[test]
fn the_export_splits_a_batch_exactly_where_the_run_does() {
    let core = Arc::new(apt_core_with_removal_args(&["remove", "-y"]));
    let installable = GenericInstallable { core };

    let many: Vec<String> = (0..BATCH_MAX_NAMES + 50)
        .map(|i| format!("pkg{i}"))
        .collect();
    let commands = installable
        .planned_removals(&specs(&many))
        .expect("a removal verb exists");

    assert_eq!(
        commands.len(),
        2,
        "{} names at a cap of {} is two commands, not one and not {}+",
        many.len(),
        BATCH_MAX_NAMES,
        many.len()
    );
    let names_of = |c: &PlannedInstall| -> Vec<String> {
        c.args
            .iter()
            .skip_while(|a| a.as_str() != "--")
            .skip(1)
            .cloned()
            .collect()
    };
    let first = names_of(&commands[0]);
    assert_eq!(
        first.len(),
        BATCH_MAX_NAMES,
        "the first command does not take the full count cap"
    );
    assert_eq!(
        commands.iter().flat_map(names_of).collect::<Vec<_>>(),
        many,
        "the export lost or reordered a name across the split"
    );
}

/// **A manager with no uninstall verb answers `None`, and that is the whole point of the type.**
///
/// `None` is what the exporter turns into a named omission. Returning `Some(vec![])` here would
/// be the silent drop that let a declared removal vanish from the artifact in the first place, so
/// the empty-list answer is asserted against explicitly.
#[test]
fn a_manager_with_no_removal_says_so_rather_than_returning_nothing_to_do() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let mut core = apt_like_core(mock, vfs);
    core.config.remove_args = vec![];
    core.config.remove_binary = None;
    let installable = GenericInstallable {
        core: Arc::new(core),
    };

    let answer = installable.planned_removals(&specs(&["snap".to_string()]));
    assert!(
        answer.is_none(),
        "a manager that cannot remove anything must answer None, so the exporter names the hole; \
         Some({:?}) would read as 'nothing to do' and drop the line",
        answer
    );
}
