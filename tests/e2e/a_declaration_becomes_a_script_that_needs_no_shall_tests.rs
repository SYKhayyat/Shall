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

/// A `cargo` on `PATH` that answers nothing and succeeds — enough for the backend to be
/// available, which is all the exporter asks of it. `install_commands` is pure argv building.
fn stub_cargo(f: &Fixture) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let stub = f.root.join("stub-bin");
    std::fs::create_dir_all(&stub).unwrap();
    let script = stub.join("cargo");
    std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    stub
}

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
fn a_scan_is_not_answered_with_the_declarations() {
    let f = Fixture::new("67-scan");
    let stub = declaring(&f);
    f.write_module("cargo:ripgrep\n");

    // `--from-scan` is #66's half and is not built: the one thing it must not do is export the
    // declaration files and let the caller believe a scan happened.
    let (out, code) = f.run_with_path(&stub, &["export", "--from-scan", "--format", "bash"]);
    assert_eq!(code, 1, "an unimplemented source exited {code}:\n{out}");
    assert!(
        out.contains("--from-scan") && out.contains("not built yet"),
        "the refusal did not name the source and why:\n{out}"
    );
    assert!(
        !f.root.join("bootstrap.sh").exists(),
        "a scan export wrote a script"
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
