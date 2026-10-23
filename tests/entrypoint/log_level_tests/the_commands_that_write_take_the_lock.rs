use super::*;

/// The direction that matters for correctness, driven through clap rather than asserted
/// about a list of strings: a command Shall cannot run without writing takes the lock.
#[test]
fn the_commands_that_write_take_the_lock() {
    for argv in [
        vec!["shall", "sync"],
        vec!["shall", "install", "apt:jq"],
        vec!["shall", "uninstall", "apt:jq"],
        vec!["shall", "adopt"],
        vec!["shall", "heal"],
        vec!["shall", "rollback", "HEAD"],
        vec!["shall", "init"],
        vec!["shall", "purge-undeclared"],
        vec!["shall", "remove-orphans"],
        vec!["shall", "rebuild"],
        vec!["shall", "apply", "shall-plan.json"],
    ] {
        let cli = Cli::parse_from(&argv);
        assert!(
            cli.command.writes(),
            "`{}` writes state and must take the data lock",
            argv[1]
        );
    }

    for argv in [
        vec!["shall", "plan"],
        vec!["shall", "list"],
        vec!["shall", "why", "apt:jq"],
        // The two the old list got wrong, in opposite directions.
        vec!["shall", "history"],
        vec!["shall", "fleet"],
        // The three that were still wrong after those two were fixed. Each writes state
        // and each is unbounded in time, so each takes the lock at the write instead
        // (`LockScope::Deferred`). `watch` is the sharp one: it never returns.
        vec!["shall", "watch"],
        vec!["shall", "shell"],
        vec!["shall", "run", "true"],
        vec!["shall", "self-upgrade"],
    ] {
        let cli = Cli::parse_from(&argv);
        assert!(
            !cli.command.writes(),
            "`{}` must not hold the 120-second exclusive lock for its whole run",
            argv[1]
        );
    }
}
