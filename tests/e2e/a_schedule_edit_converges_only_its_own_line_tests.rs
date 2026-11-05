//! `schedule add` / `schedule remove` converge the schedule they named, and nothing else
//! (#130 — the `C7` class on the verb the ruling did not cover).
//!
//! `handle_schedule` edited a line and ended in a whole-machine `handle_sync`, so removing one
//! timer converged the rest of the configuration on a machine the user had just asked to take
//! one thing off. `C7` ruled that shape out for the package verbs and left this one filed rather
//! than guessed at, because a `schedule:` is a *resource* and its converge is the non-package
//! phases, not the planner's package keys.
//!
//! **`dir:` is the witness.** It needs no root, no manager and no network, and whether the
//! directory exists is the entire assertion. It is also the thing a whole-machine converge
//! would create while a narrowed run must leave alone.
//!
//! **Why the removal is driven for real and the addition is driven against a stub.** `schedule
//! remove` with no ledger row touches no scheduler at all (`Extras::undo_one` says so and warns),
//! so it is hermetic on every host. A real `add` provisions through systemd/launchd/Task
//! Scheduler; the test stands the platform's own scheduler program in on `PATH`, so the
//! provisioner really runs and writes its record without reaching this machine's session.
//!
//! **The record it writes is the platform's, and the assertion asks the platform.** This test
//! used to name systemd's `.timer` path on every Unix, so it passed on Linux and failed on macOS
//! — where `schedule add` writes `~/Library/LaunchAgents/com.shall.<name>.plist` and never a
//! unit. That is `#130`'s own "unproven half" arriving as a red Apple job rather than a report.

use crate::harness::{decl, Fixture};

fn dir_line(path: &std::path::Path) -> String {
    format!("dir:{}", decl(path))
}

fn schedules(f: &Fixture) -> String {
    std::fs::read_to_string(f.cfg().join("schedules")).unwrap_or_default()
}

const TIDY: &str = "schedule:tidy {\n  cron = 0 3 * * *\n  run  = check\n}\n";

/// One pending declaration in the config, one schedule removed by name. The schedule goes; the
/// pending declaration stays pending, because converging it is `sync`'s job.
#[test]
fn a_schedule_removal_does_not_converge_the_machine() {
    let f = Fixture::new("c7-schedule-remove");
    let pending = f.root.join("pending-dir");
    f.write_module(&format!("{}\n", dir_line(&pending)));
    f.write("schedules", TIDY);

    let (out, code) = f.run(&["schedule", "remove", "tidy"]);
    assert_eq!(code, 0, "`schedule remove` failed:\n{out}");
    assert!(
        !schedules(&f).contains("schedule:tidy"),
        "`schedule remove` left the line it was asked to remove:\n{}\n{out}",
        schedules(&f)
    );
    assert!(
        !pending.is_dir(),
        "`schedule remove` converged the rest of the config — the pending `dir:` was created. \
         A whole-machine converge is `sync`'s (#130):\n{out}"
    );
}

/// The scheduler program this platform's provisioner drives, and the verb that arms a task.
///
/// `Scheduler::detect` chooses the provisioner by `cfg!`, so the test has to choose its stub the
/// same way. Naming `systemctl` everywhere is what made the Apple job red: on macOS the program
/// is `launchctl` and the arming verb is `load`.
#[cfg(target_os = "linux")]
fn platform_scheduler() -> (&'static str, &'static str) {
    ("systemctl", "enable")
}
#[cfg(target_os = "macos")]
fn platform_scheduler() -> (&'static str, &'static str) {
    ("launchctl", "load")
}

/// Where the platform's provisioner writes the record that the schedule exists.
///
/// Linux: `$XDG_CONFIG_HOME/systemd/user/shall-<name>.timer` (the fixture pins `XDG_CONFIG_HOME`
/// at its own root). macOS: `$HOME/Library/LaunchAgents/com.shall.<name>.plist`.
#[cfg(target_os = "linux")]
fn provisioned_record(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    root.join(".config")
        .join("systemd")
        .join("user")
        .join(format!("shall-{name}.timer"))
}
#[cfg(target_os = "macos")]
fn provisioned_record(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    root.join("Library")
        .join("LaunchAgents")
        .join(format!("com.shall.{name}.plist"))
}

/// The addition is narrowed the same way, driven for real against the platform's scheduler.
///
/// The stub records what it was asked and exits 0, so the schedule phase really runs — record
/// written, arming verb issued — without reaching this machine's session. The pending `dir:`
/// must not be created at all.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn a_schedule_addition_applies_its_own_line_and_no_other() {
    use std::os::unix::fs::PermissionsExt;

    let f = Fixture::new("c7-schedule-add");
    let pending = f.root.join("pending-dir");
    f.write_module(&format!("{}\n", dir_line(&pending)));

    let (program, arming_verb) = platform_scheduler();
    let stub = f.root.join("stub-bin");
    std::fs::create_dir_all(&stub).unwrap();
    let log = f.root.join("scheduler.log");
    let script = stub.join(program);
    std::fs::write(
        &script,
        format!("#!/bin/sh\necho \"$@\" >> '{}'\nexit 0\n", log.display()),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

    let (out, code) = f.run_with_path(
        &stub,
        &[
            "-y",
            "schedule",
            "add",
            "tidy",
            "--cron",
            "0 3 * * *",
            "--run",
            "check",
        ],
    );
    assert_eq!(
        code,
        0,
        "`schedule add` failed:\n{out}\n--- schedules:\n{}",
        schedules(&f)
    );
    assert!(
        schedules(&f).contains("schedule:tidy"),
        "`schedule add` wrote no line:\n{}",
        schedules(&f)
    );
    let record = provisioned_record(&f.root, "tidy");
    assert!(
        record.is_file(),
        "the schedule it named was not provisioned: {} is absent. A narrowed run still applies \
         what it was asked for:\n{out}",
        record.display()
    );
    let asked = std::fs::read_to_string(&log).unwrap_or_default();
    assert!(
        asked.contains(arming_verb),
        "the record was written and never armed ({arming_verb}) — the stub `{program}` was \
         asked:\n{asked}"
    );
    assert!(
        !pending.is_dir(),
        "`schedule add` converged the rest of the config — the pending `dir:` was created. A \
         whole-machine converge is `sync`'s (#130):\n{out}"
    );
}

/// Taking a line out by hand and then asking the verb to remove it is not an error: the machine
/// half is still the verb's business, and the answer says the line was not there. What must not
/// happen is the whole config converging on the way past.
#[test]
fn a_schedule_removal_of_a_line_that_is_already_gone_still_does_not_converge() {
    let f = Fixture::new("c7-schedule-remove-absent");
    let pending = f.root.join("pending-dir");
    f.write_module(&format!("{}\n", dir_line(&pending)));

    let (out, code) = f.run(&["schedule", "remove", "tidy"]);
    assert_eq!(
        code, 0,
        "removing a name that is not declared failed:\n{out}"
    );
    assert!(
        !pending.is_dir(),
        "removing a name that is not declared converged the whole config:\n{out}"
    );
}
