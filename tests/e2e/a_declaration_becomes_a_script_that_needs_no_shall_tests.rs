//! `shall export --from-declaration --format bash` (#67, II.94) — a script that reproduces the
//! machine without Shall.
//!
//! **What these tests are for.** The artifact's whole value is that its commands are the ones a
//! sync would really run, so the assertions are on argv, not on "a file was written". The suite
//! is built by `cargo`, but a fixture that depends on the developer's `cargo` being on `PATH`
//! would be measuring the machine: `cargo` stands in on `PATH` here, as `restic` does for the
//! invariant tests.
//!
//! The four failures worth catching: a script that does not say what a sync says, one that is
//! written anyway when it cannot say everything, one that touches the machine it describes, and
//! a source flag that quietly answers the other source's question.

use crate::harness::Fixture;

/// The one unix-only import in this file, in a module no other platform can reach.
///
/// **It was at file scope, and that is why the whole test target failed to build on Windows.**
/// `PermissionsExt` does not exist there, so `tests/e2e/…::a_declaration…` and the invariant file
/// next door were E0433/E0599 errors rather than four tests being skipped — and a suite target
/// that does not compile takes every test in it, on that row, with it (#168).
#[cfg(unix)]
mod hosting {
    use crate::harness::Fixture;

    pub(super) fn stub_cargo(f: &Fixture) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let stub = f.root.join("stub-bin");
        std::fs::create_dir_all(&stub).unwrap();
        let script = stub.join("cargo");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        // The exporter asks the manager whether it is *there*; on unix "there" for a stub is
        // "executable", so this line is load-bearing and not decoration.
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        stub
    }

    /// The same stub shape as `stub_cargo`, for the manager a removal is declared against.
    pub(super) fn stub_apt(stub: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;

        let script = stub.join("apt");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[cfg(unix)]
use hosting::{stub_apt, stub_cargo};

#[cfg(unix)]
fn declaring(f: &Fixture) -> std::path::PathBuf {
    f.write("priority", "cargo\n");
    stub_cargo(f)
}

#[cfg(unix)]
#[test]
fn the_script_carries_the_command_a_sync_would_run() {
    let f = Fixture::new("67-bash");
    let stub = declaring(&f);
    f.write_module("cargo:ripgrep\ncargo:pv\n");

    let (out, code) = f.run_with_path(&stub, &["export", "--from-declaration", "--format", "bash"]);
    assert_eq!(code, 0, "the export refused a config it can carry:\n{out}");

    let script = std::fs::read_to_string(f.root.join("bootstrap.sh")).unwrap();
    assert!(
        script.contains("cargo install -- pv ripgrep"),
        "the script does not carry the argv a sync runs:\n{script}"
    );
    assert!(
        script.starts_with("#!/usr/bin/env bash"),
        "the artifact is not a script:\n{script}"
    );
    // The command has to be runnable by the shell the shebang names.
    let check = std::process::Command::new("bash")
        .arg("-n")
        .arg(f.root.join("bootstrap.sh"))
        .status()
        .unwrap();
    assert!(
        check.success(),
        "the generated script does not parse:\n{script}"
    );
}

#[cfg(unix)]
#[test]
fn a_line_the_format_cannot_carry_is_refused_and_nothing_is_written() {
    let f = Fixture::new("67-refuse");
    let stub = declaring(&f);
    f.write_module("cargo:ripgrep\nlink:./vimrc @target=~/.vimrc\n");
    f.write("vimrc", "set number\n");

    let (out, code) = f.run_with_path(&stub, &["export", "--from-declaration", "--format", "bash"]);
    assert_eq!(
        code, 2,
        "an export that cannot say everything exited {code}:\n{out}"
    );
    assert!(
        out.contains("resource lines are not exported yet"),
        "the refusal did not say what it could not carry:\n{out}"
    );
    assert!(
        !f.root.join("bootstrap.sh").exists(),
        "a refused export wrote a script anyway"
    );
}

#[cfg(unix)]
#[test]
fn best_effort_writes_the_script_and_still_says_it_is_incomplete() {
    let f = Fixture::new("67-best-effort");
    let stub = declaring(&f);
    f.write_module("cargo:ripgrep\nlink:./vimrc @target=~/.vimrc\n");
    f.write("vimrc", "set number\n");

    let (out, code) = f.run_with_path(
        &stub,
        &[
            "export",
            "--from-declaration",
            "--format",
            "bash",
            "--best-effort",
        ],
    );
    // The file exists AND the run says it is not complete: one answer, and it must not be
    // mistaken for the other (II.94).
    assert_eq!(code, 2, "a lossy export exited {code}:\n{out}");
    let script = std::fs::read_to_string(f.root.join("bootstrap.sh")).unwrap();
    assert!(
        script.contains("cargo install -- ripgrep"),
        "the script lost the lines it could carry:\n{script}"
    );
    assert!(
        script.contains("What this script does NOT do"),
        "the hole was not marked in the artifact:\n{script}"
    );
    assert!(
        script.contains("~/.vimrc") || script.contains("link:"),
        "the marked hole does not name the line:\n{script}"
    );
}

#[cfg(unix)]
#[test]
fn a_scan_reports_what_the_machine_has_not_what_the_files_say() {
    let f = Fixture::new("67-scan-real");
    let stub = declaring(&f);
    // The declarations ask for `pv`; the machine has `ripgrep` installed. A scan must report the
    // machine — an export that answered with the declaration files would be `--from-declaration`
    // wearing a mask.
    f.write_module(
        "cargo:pv
",
    );
    std::fs::write(
        stub.join("cargo"),
        "#!/bin/sh\ncase \"$1\" in\n  install)\n    if [ \"$2\" = \"--list\" ]; then\n      printf 'ripgrep v14.1.0:\\n    rg\\n'\n      exit 0\n    fi\n    exit 0;;\n  *) exit 0;;\nesac\n",
    )
    .unwrap();

    let (out, code) = f.run_with_path(&stub, &["export", "--from-scan", "--format", "bash"]);
    assert_eq!(code, 0, "a scan with findings exited {code}:\n{out}");
    let script = std::fs::read_to_string(f.root.join("bootstrap.sh")).unwrap();
    assert!(
        script.contains("cargo install -- ripgrep"),
        "the scan script is missing the discovered package:\n{script}"
    );
    assert!(
        !script.contains("pv"),
        "the scan script carried a declaration the machine does not have:\n{script}"
    );
    assert!(
        script.contains("from-scan"),
        "the artifact does not say it came from a scan:\n{script}"
    );
}

#[cfg(unix)]
#[test]
fn a_scan_that_finds_nothing_writes_nothing() {
    let f = Fixture::new("67-scan-empty");
    let stub = declaring(&f);
    f.write_module(
        "cargo:pv
",
    );
    // The machine has nothing `shall adopt` would claim: an empty script would read as "nothing
    // to reproduce" when really nothing was found.
    std::fs::write(stub.join("cargo"), "#!/bin/sh\nexit 0\n").unwrap();

    let (out, code) = f.run_with_path(&stub, &["export", "--from-scan", "--format", "bash"]);
    assert_eq!(code, 2, "an empty scan exited {code}:\n{out}");
    assert!(
        !f.root.join("bootstrap.sh").exists(),
        "an empty scan wrote a script"
    );
}

#[cfg(unix)]
#[test]
fn the_export_never_touches_the_machine_it_describes() {
    let f = Fixture::new("67-no-writes");
    let stub = declaring(&f);
    f.write_module("cargo:ripgrep\n");
    // A `cargo` that records every call it gets. `planned_installs` is argv building, so an
    // export must reach it zero times — the moment the exporter runs the command to describe it,
    // the artfact has installed something.
    let log = f.root.join("cargo.log");
    std::fs::write(
        stub.join("cargo"),
        format!("#!/bin/sh\necho \"$@\" >> '{}'\nexit 0\n", log.display()),
    )
    .unwrap();

    let (out, code) = f.run_with_path(&stub, &["export", "--from-declaration", "--format", "bash"]);
    assert_eq!(code, 0, "the export failed:\n{out}");
    let calls = std::fs::read_to_string(&log).unwrap_or_default();
    assert!(
        !calls.contains("install"),
        "the export ran the install it was describing:\n{calls}"
    );
}

#[cfg(unix)]
#[test]
fn an_opted_out_line_is_not_a_hole_and_not_an_install() {
    let f = Fixture::new("67-opt-out");
    let stub = declaring(&f);
    // The link is a resource line this artifact cannot carry — and its owner said so, once, in
    // the file. That is not a hole: the export is complete as far as its config asks (II.94).
    f.write_module("cargo:ripgrep\nlink:./vimrc @target=~/.vimrc,export=no\n");
    f.write("vimrc", "set number\n");

    let (out, code) = f.run_with_path(&stub, &["export", "--from-declaration", "--format", "bash"]);
    assert_eq!(code, 0, "an opted-out line made the export refuse:\n{out}");
    let script = std::fs::read_to_string(f.root.join("bootstrap.sh")).unwrap();
    assert!(
        !script.contains("NOT do"),
        "the opted-out line was reported as a hole:\n{script}"
    );
    assert!(script.contains("cargo install -- ripgrep"), "{script}");
}

/// **An `absent:` line is a sentence the artifact has to carry, not one it may swallow (#67).**
///
/// The exporter filtered the resolved specs on `present`, which is right for building the
/// installs and wrong for everything else: an `absent:apt:snap` line reached neither the install
/// map nor the omission list, so the script came out describing a machine that still had snap on
/// it while the declaration said otherwise. Nothing in the output distinguished that from a
/// declaration that never mentioned snap — the silent-nothing class the export's own refusal
/// exists to prevent.
///
/// The assertion is on argv rather than on the word "remove", because "the script mentions snap"
/// is satisfied by an install of it, which is the exact inversion that is worse than silence.
#[cfg(unix)]
#[test]
fn a_declared_removal_reaches_the_script_as_a_removal() {
    let f = Fixture::new("67-absent");
    let stub = declaring(&f);
    f.write("priority", "cargo\napt\n");
    // A stub `apt` so the backend is available on any host; `planned_removals` builds argv and
    // runs nothing, exactly as the cargo stub above does for installs.
    stub_apt(&stub);
    f.write_module("cargo:ripgrep\nabsent:apt:snap\n");

    let (out, code) = f.run_with_path(&stub, &["export", "--from-declaration", "--format", "bash"]);
    assert_eq!(
        code, 0,
        "the export refused a config it can carry (a removal has an argv here):\n{out}"
    );

    let script = std::fs::read_to_string(f.root.join("bootstrap.sh")).unwrap();
    assert!(
        script.contains("apt remove") && script.contains("snap"),
        "the declared removal is missing from the artifact, so the script reproduces a machine \
         the declaration does not describe:\n{script}"
    );
    assert!(
        !script.contains("apt install") || !script.contains("snap"),
        "the declaration asked for snap's ABSENCE and the script installs it:\n{script}"
    );

    // A bootstrap leaves what it installed in place if the removal fails, so the removal has to
    // come after the installs rather than before them.
    let install_at = script
        .find("install")
        .expect("the install line is in the script");
    let remove_at = script
        .find("remove")
        .expect("the removal line is in the script");
    assert!(
        remove_at > install_at,
        "the removal runs before the install, so a failed install leaves the machine with \
         neither:\n{script}"
    );
}
