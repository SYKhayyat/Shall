//! An argument that is not UTF-8 is refused, never fatal.
//!
//! **What was found.** `main` read argv with `std::env::args()`, which unwraps the conversion,
//! so the first non-UTF-8 argument aborted the process: under `panic = "abort"` that is a core
//! dump rather than a message. Three doors reach it — a package name, a path given to `-c` or
//! `--data-dir`, and a shim's forwarded arguments — and the shim's is the worst: a shim exists
//! to be transparent, and `jq` behind one could not be handed a Latin-1 filename that the real
//! `jq` accepts, though the shim never even got as far as the target.
//!
//! **The exit code is 1, not 2.** clap's own convention for a usage error is 2, and `U21`
//! spends 2 on *a read-only command looked and found work to do* — so a job branching on the
//! published table would read a mangled argument as a drifted machine. `Q3` ruled a usage
//! error exits 1 and the table stays at four codes; this is that ruling applied to a new door.
//!
//! Unix only: Windows argv is UTF-16, where every `OsString` has a `String` behind it and the
//! conversion cannot fail.

#![cfg(unix)]

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// One argument that is not valid UTF-8, as `argv` can really carry it.
fn undecodable() -> OsString {
    OsString::from(OsStr::from_bytes(b"\xff"))
}

/// The real binary, with no config, no data directory and no stdin of its own.
///
/// **A home of its own, not just a data directory.** `locate` falls back to
/// `dirs::config_dir()/shall.settings.toml`, so a test that only redirects `SHALL_DATA_DIR`
/// reads the developer's real settings file — and a shim with no declaration then resolves its
/// own name against a real package manager. That is a test reaching the network to check an
/// argv conversion.
fn isolated(dir: &Path, program: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir.join("xdg-config"))
        .env("XDG_DATA_HOME", dir.join("xdg-data"))
        .env("SHALL_DATA_DIR", dir.join("data"))
        .env_remove("SHALL_CONFIG_DIR")
        .current_dir(dir)
        .stdin(Stdio::null());
    command
}

fn shall(dir: &Path) -> Command {
    isolated(dir, Path::new(env!("CARGO_BIN_EXE_shall")))
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The panic signature `std::env::args()` produces: a Rust panic, which `panic = "abort"`
/// turns into SIGABRT in a release build and exit 101 in a debug one.
fn assert_did_not_panic(out: &Output) {
    let text = stderr_of(out);
    assert!(
        !text.contains("panicked at"),
        "a non-UTF-8 argument panicked the process instead of being refused:\n{text}"
    );
    assert_ne!(
        out.status.code(),
        Some(101),
        "the unwrap on the argv conversion is still there:\n{text}"
    );
}

#[test]
fn a_non_utf8_subcommand_argument_is_a_usage_error() {
    let dir = tempfile::TempDir::new().unwrap();
    let out = shall(dir.path())
        .args([OsString::from("search"), undecodable()])
        .output()
        .expect("the binary should run");

    assert_did_not_panic(&out);
    let text = stderr_of(&out);
    assert!(
        text.to_lowercase().contains("utf-8"),
        "the refusal must name what it could not read, and `search` takes a string:\n{text}"
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "Q3: a usage error exits 1 — 2 is reserved for a read-only command that found work:\n{text}"
    );
}

/// The door the report named as worst: a shim re-entering Shall with somebody else's argv.
#[test]
fn a_shim_reads_a_non_utf8_argument_without_dying_on_it() {
    let dir = tempfile::TempDir::new().unwrap();
    let shim = dir.path().join("not-shall");
    std::fs::copy(env!("CARGO_BIN_EXE_shall"), &shim).expect("copy the binary as a shim");

    let out = isolated(dir.path(), &shim)
        .arg(undecodable())
        .output()
        .expect("the shim should run");

    // It has nowhere to forward to — no repo, no declaration — so it fails. What it must not
    // do is die on the way in, which is what the argv conversion did.
    assert_did_not_panic(&out);
    assert!(
        stderr_of(&out).contains("priority"),
        "the shim did not get as far as reading a configuration, so this proves nothing about \
         its argv:\n{}",
        stderr_of(&out)
    );
}

/// `--config` names a path, and a path is bytes. This door is read from argv *before* clap
/// parses, for the alias table, so a `String` there aborted the process before any command ran.
#[test]
fn a_config_path_that_is_not_utf8_is_opened_by_its_bytes() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join(OsStr::from_bytes(b"pref-\xff.toml"));
    // Deliberately not valid: the refusal below is the proof that this file — and not the
    // default location — is the one that was opened. A `String` conversion never gets here.
    std::fs::write(&path, b"max_parallel = \"four\"\n").unwrap();

    let out = shall(dir.path())
        .arg("--config-dir")
        .arg(dir.path())
        .arg("--config")
        .arg(&path)
        .arg("plan")
        .output()
        .expect("the binary should run");

    assert_did_not_panic(&out);
    let text = stderr_of(&out);
    assert!(
        text.contains("is not valid TOML"),
        "the file named by a non-UTF-8 path was not the one read:\n{text}"
    );
    assert!(
        text.contains("pref-"),
        "the refusal must name the file it read:\n{text}"
    );
}
