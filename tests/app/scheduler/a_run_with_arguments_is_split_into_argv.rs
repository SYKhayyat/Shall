//! **A `run` with arguments survives launchd** (#195).
//!
//! `run` is the arguments the scheduler hands to Shall, and the three renderers deliver them
//! differently: the systemd unit and the `schtasks` command line are *lines* the OS splits, and
//! launchd's `ProgramArguments` goes to `execve` whole. So a schedule with `run = sync --dry-run`
//! worked on Linux and Windows and arrived on macOS as one argument — `argv[1] ==
//! "sync --dry-run"` — and died in argument parsing on every firing, with nothing anywhere
//! saying why. That is the same shape as the unit that asked Shall to run a subcommand called
//! `shall`, which this module already has a ruling about, one space away.
//!
//! The negative control matters as much as the fix: the two line renderers must **keep** their
//! single-line spelling. A fix that split them too, or that quoted the words, would be a
//! different bug wearing this one's clothes.

use super::*;

fn with_command(command: &str) -> ScheduleConfig {
    ScheduleConfig {
        command: command.into(),
        ..cfg("0 2 * * *")
    }
}

/// Every word of the `run` is its own `<string>`, and no element holds a space that was a
/// separator in the declaration.
#[test]
fn a_run_with_arguments_becomes_one_argv_element_per_word() {
    let plist = MacLaunchdProvisioner.plist(
        &with_command("sync --dry-run"),
        Path::new("/usr/local/bin/shall"),
        &here(),
    );

    assert!(
        plist.contains("<string>sync</string><string>--dry-run</string>"),
        "the arguments are not separate argv elements, so launchd would hand Shall one word:\n{plist}"
    );
    assert!(
        !plist.contains("<string>sync --dry-run</string>"),
        "the whole command is still one argument:\n{plist}"
    );

    // The binary stays first and the words follow it, in order. The array ends where `</array>`
    // says, or the log paths below it are strings too.
    let args = plist
        .split("<key>ProgramArguments</key>")
        .nth(1)
        .expect("the plist has a ProgramArguments array");
    let args = args.split("</array>").next().expect("the array is closed");
    let words: Vec<&str> = args
        .split("<string>")
        .skip(1)
        .map(|w| w.split("</string>").next().unwrap_or(""))
        .collect();
    assert_eq!(
        words,
        vec!["/usr/local/bin/shall", "sync", "--dry-run"],
        "argv is built in the wrong order or with the wrong words: {words:?}"
    );
}

/// A program path with a space in it is **one** word, and the quotes are not part of it — the
/// same split `refuse_self_named` asks for the first word, so the two cannot disagree about
/// where that word ends.
#[test]
fn a_quoted_program_path_is_one_word_and_keeps_its_space() {
    let plist = MacLaunchdProvisioner.plist(
        &with_command("\"/opt/Shall Beta/shall\" sync"),
        Path::new("/opt/Shall Beta/shall"),
        &here(),
    );
    assert!(
        plist.contains("<string>/opt/Shall Beta/shall</string><string>sync</string>"),
        "a quoted path split at its space, or kept its quotes:\n{plist}"
    );
}

/// The line renderers keep their spelling: systemd and `schtasks` are handed a line and split it
/// themselves, so a fix that made them emit argv would be a different bug.
#[test]
fn the_line_renderers_keep_the_command_as_one_line() {
    let config = with_command("sync --dry-run");
    let bin = Path::new("/usr/local/bin/shall");

    let unit = LinuxSystemdProvisioner::service_unit(&config, bin, &here());
    assert!(
        unit.contains("ExecStart=/usr/local/bin/shall sync --dry-run"),
        "the systemd unit stopped writing the command as a line:\n{unit}"
    );

    let line = WindowsTaskProvisioner::command_line(&config, bin, &here());
    assert!(
        line.ends_with("sync --dry-run"),
        "the schtasks command line stopped writing the command as a line: {line}"
    );
}

/// Text in a plist is not markup, so a word carrying `&`, `<` or `>` has to be escaped — launchd
/// answers an unparseable plist by not running the job at all.
#[test]
fn an_argument_with_markup_in_it_is_escaped() {
    let plist = MacLaunchdProvisioner.plist(
        &with_command("sync --filter \"a&b<c\""),
        Path::new("/usr/local/bin/shall"),
        &here(),
    );
    assert!(
        plist.contains("<string>--filter</string><string>a&amp;b&lt;c</string>"),
        "an argument with markup in it was written raw:\n{plist}"
    );
}
