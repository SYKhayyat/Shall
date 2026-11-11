use super::*;

/// `run` is the arguments, and the scheduler supplies the program (owner ruling, 2026-11-10,
/// `#158`).
///
/// The unit written on the owner's machine read `ExecStart=/…/shall shall sync`, so every firing
/// died in argument parsing — `error: unrecognized subcommand 'shall'` — and nothing in the
/// operation said so. It is the same class `cron` has always refused at parse time, one option
/// over.
#[test]
fn a_run_that_names_the_binary_is_refused() {
    for run in [
        "shall sync",
        "shall",
        // The name a scheduler may carry, and the forms a full path arrives in. A path with a
        // space in it has to be quoted for the scheduler to see one program, and the quote is
        // part of the word the split sees.
        "shall.exe sync",
        "/usr/local/bin/shall sync",
        "'C:\\bin\\shall.exe' sync",
        "\"C:\\Program Files\\Shall\\shall.exe\" sync",
    ] {
        let o = opts(&[("cron", "0 2 * * *"), ("run", run)]);
        let err = schedule_config("nightly", &o, &origin(), &shipped())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("already runs itself"),
            "`run = {run}` was accepted, and the unit it writes can never run: {err}"
        );
        assert!(
            err.contains("run = sync"),
            "the error does not say what to write instead: {err}"
        );
    }
}

/// The control, and the reason the rule reads the word and not the line: `shall-helper` is a
/// different program, and a command that merely mentions the name is not the program itself.
#[test]
fn a_command_that_merely_starts_with_the_name_is_not_refused() {
    for run in [
        "sync",
        "shall-notify sync",
        "notify-shall sync",
        "'my tools/shall-helper' status",
    ] {
        let o = opts(&[("cron", "0 2 * * *"), ("run", run)]);
        let cfg = schedule_config("nightly", &o, &origin(), &shipped())
            .unwrap_or_else(|e| panic!("`run = {run}` was refused and it should not be: {e}"));
        assert_eq!(cfg.command, run);
    }
}
