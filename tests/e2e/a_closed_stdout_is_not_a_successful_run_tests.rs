//! A closed stdout is an exit, not a silent success.
//!
//! **What was found.** `println!` panics when its write fails, and the release profile is
//! `panic = "abort"` — so `shall sync | true` (a `tee` that died, an SSH session that dropped)
//! aborted the process. `main`'s hook caught that panic and left with **0**, whatever the run
//! had been doing, and `std::process::exit` runs no destructor on the way out. Two things
//! therefore went wrong at once: a mutating run reported success, and the data lock's owner
//! stamp outlived the process — so every later reader saw a writer that was not there.
//!
//! **Why the status depends on the run.** `shall search x | head` is the case the hook was
//! written for: the wanted answer was delivered and `0` is right. `shall init | true` is the
//! opposite: the answer was cut off halfway and the files it was writing were never reported.
//! One status for both is the defect.
//!
//! **The pipe is closed before the child starts.** A `Stdio::piped()` handle dropped after the
//! spawn is a race — a small report fits in the pipe buffer and never sees `EPIPE` — so the
//! child's stdout is a socket whose other end is already gone. The first `println!` refuses.

#![cfg(unix)]

use std::path::Path;
use std::process::{Command, Output, Stdio};

fn isolated(dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_shall"));
    command
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir.join("xdg-config"))
        .env("XDG_DATA_HOME", dir.join("xdg-data"))
        .env("SHALL_DATA_DIR", dir.join("data"))
        .env_remove("SHALL_CONFIG_DIR")
        .arg("--data-dir")
        .arg(dir.join("data"))
        .current_dir(dir)
        .stdin(Stdio::null());
    command
}

/// Run with a stdout whose reader has already gone.
fn with_a_dead_reader(command: &mut Command) -> Output {
    let (reader, writer) = std::os::unix::net::UnixStream::pair().unwrap();
    drop(reader);
    let writer = std::os::unix::io::OwnedFd::from(writer);
    let child = command
        .stdout(Stdio::from(writer))
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary should start");
    child.wait_with_output().expect("the binary should finish")
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// A run that writes reports the cut-off; its lock's stamp goes with it.
#[test]
fn a_writing_run_that_loses_its_reader_fails_and_leaves_no_writer_stamp() {
    let dir = tempfile::TempDir::new().unwrap();
    let out = with_a_dead_reader(isolated(dir.path()).arg("init"));

    assert_eq!(
        out.status.code(),
        Some(1),
        "a scaffolding run cut off halfway reported success:\n{}",
        stderr_of(&out)
    );
    assert!(
        !stderr_of(&out).contains("panicked at"),
        "the panic reached the default hook, so the process aborted instead of leaving:\n{}",
        stderr_of(&out)
    );
    assert!(
        !dir.path().join("data/shall.lock.owner").exists(),
        "the lock's owner stamp outlived the process, so every reader now sees a writer that \
         is not there"
    );
}

/// **The third panic shape: a dependency's own `expect()`.**
///
/// `clap_complete::generate` writes to the handle it is handed and `expect()`s each write, so
/// `shall completions bash | head -1` panicked with `failed to write completion file: … Broken
/// pipe`. That message is not the standard library's "failed printing to stdout", so the hook
/// did not answer it and the release profile's `panic = "abort"` made it a SIGABRT — a core dump
/// on the operator's machine for every harness pipe that closed on the first line, which is what
/// the desktop notification was about. The completions are generated into a buffer and printed
/// now, so this is the same closed stdout the rest of the program already answers.
#[test]
fn completions_that_lose_their_reader_is_an_exit_and_not_a_core_dump() {
    let dir = tempfile::TempDir::new().unwrap();
    let out = with_a_dead_reader(isolated(dir.path()).args(["completions", "bash"]));

    assert!(
        !stderr_of(&out).contains("failed to write completion file"),
        "the generator's own `expect()` reached the default hook, so this aborted instead of \
         leaving an exit code:\n{}",
        stderr_of(&out)
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "`shall completions bash | head -1` is a reading run whose reader went away: it must \
         leave 0, not be killed:\n{}",
        stderr_of(&out)
    );
}

/// The case the hook was written for keeps its quiet exit.
#[test]
fn a_reading_run_that_loses_its_reader_is_still_success() {
    let dir = tempfile::TempDir::new().unwrap();
    // A repo to read; the control run also proves the scaffold itself succeeds normally.
    let control = isolated(dir.path())
        .arg("init")
        .output()
        .expect("the binary should run");
    assert_eq!(
        control.status.code(),
        Some(0),
        "the control run failed, so the closed-pipe assertion below would mean nothing:\n{}",
        stderr_of(&control)
    );

    let out = with_a_dead_reader(isolated(dir.path()).arg("plan"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "`shall plan | head` is the case this hook exists for and must stay quiet:\n{}",
        stderr_of(&out)
    );
}
