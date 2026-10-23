use super::*;

/// The lock classification, asked of the enum and of clap — the two things that cannot
/// drift from each other.
///
/// **The `undo` disease, found in the lock list this replaces.** Twelve of its thirty-three
/// entries named commands the program does not have — `status` (now `check drift`),
/// `doctor`, `unmanaged`, `absent`, `insight`, `show`, `audit`, `outdated`, `log`, `locate`,
/// `metrics`, `verify`. Two tests guarded it and **both guarded invention**: that every name
/// on the list was real. Nothing guarded omission or misclassification, which is the half
/// that costs an entry out of `registry.json` — and both were live. `history` was exempt
/// while reaching `handle_rollback` → `handle_sync`, the entire install/remove path, and
/// `fleet` was absent from the list while touching no local state at all.
///
/// **Three arms now, and the third is the one that was missing.** `Reader` never takes the
/// lock; `Writer` holds it for the whole run; `Deferred` writes state but takes the lock at
/// each mutating action, because its duration is a person's or a loop's. `watch`, `shell`
/// and `run` were `Writer` — and `watch` never returns, so the documented GitOps deployment
/// held the exclusive lock for the life of the daemon.
#[test]
fn the_readers_are_exactly_the_commands_that_read() {
    use clap::CommandFactory;
    use shall::cli::LockScope;

    // **Asked of the program, not of its source text.** The first version of this scanned
    // `args.rs` for the arms' variant names, because a subcommand with required arguments
    // cannot be parsed from its name alone. That made the assertion depend on where rustfmt
    // put a line break — it collapsed the four-variant arm into a block and the marker
    // stopped matching — and a gate that a formatter can silence is not a gate.
    //
    // So the argv is built out of clap's own metadata instead: every required argument gets
    // a value it will accept — the first of its `possible_values` where it has them, because
    // `shall completions filler` is not a shell — and a subcommand that carries subcommands
    // of its own recurses into its first child. That reaches all sixty-four, including
    // `repo`, `hook-record` and `completions`, which no amount of positional filler does.
    fn argv_for(sub: &clap::Command) -> Vec<String> {
        let mut argv = vec![sub.get_name().to_string()];
        for arg in sub.get_arguments() {
            if !arg.is_required_set() {
                continue;
            }
            let value = arg
                .get_possible_values()
                .first()
                .map(|v| v.get_name().to_string())
                .unwrap_or_else(|| "filler".to_string());
            match arg.get_long() {
                Some(long) => {
                    argv.push(format!("--{long}"));
                    if matches!(
                        arg.get_action(),
                        clap::ArgAction::Set | clap::ArgAction::Append
                    ) {
                        argv.push(value);
                    }
                }
                None => argv.push(value),
            }
        }
        if let Some(child) = sub.get_subcommands().next() {
            argv.extend(argv_for(child));
        }
        argv
    }

    let mut readers = std::collections::BTreeSet::new();
    let mut deferred = std::collections::BTreeSet::new();
    let mut unparsed = Vec::new();
    for sub in <Cli as CommandFactory>::command().get_subcommands() {
        let name = sub.get_name().to_string();
        let mut argv = vec!["shall".to_string()];
        argv.extend(argv_for(sub));
        match Cli::try_parse_from(&argv).map(|cli| cli.command.lock_scope()) {
            Ok(LockScope::Reader) => {
                readers.insert(name);
            }
            Ok(LockScope::Deferred) => {
                deferred.insert(name);
            }
            Ok(LockScope::Writer) => {}
            Err(e) => unparsed.push(format!(
                "{name} (as `{}`: {})",
                argv[1..].join(" "),
                e.kind()
            )),
        }
    }

    // A subcommand nothing here could parse is a subcommand this test does not classify,
    // and an unclassified subcommand is exactly the omission the old list could not see.
    assert!(
        unparsed.is_empty(),
        "{} subcommand(s) could not be driven through clap, so their lock scope was never \
         examined:\n  {}\n\nFix the argv builder rather than leaving them out.",
        unparsed.len(),
        unparsed.join("\n  ")
    );

    let expected_deferred: std::collections::BTreeSet<String> =
        ["history", "run", "self-upgrade", "shell", "watch"]
            .iter()
            .map(|s| s.to_string())
            .collect();
    assert_eq!(
        deferred, expected_deferred,
        "the set of commands that take the lock at the write rather than for the run \
         changed. A command belongs here when its duration is decided by a person, a loop \
         or a program Shall does not own — never by the package work it performs."
    );

    let expected: std::collections::BTreeSet<String> = [
        "adapters",
        "check",
        "completions",
        "config",
        "diff",
        "edit",
        "eval",
        "export",
        "fleet",
        "info",
        "list",
        "path",
        "plan",
        "policy",
        "protected",
        "repl",
        "sbom",
        "search",
        "try",
        "vars",
        "why",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    assert_eq!(
        readers, expected,
        "the set of commands exempted from the data lock changed. Adding a WRITER is free; \
         adding a reader means claiming it never writes under `data/`, so it has to be \
         claimed here too. Not locking a writer costs an entry out of `registry.json`, \
         which is a removal."
    );
}
