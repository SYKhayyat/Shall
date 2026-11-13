//! `invariant:` (#74) — a promise the config files make, measured against the backup tool's own
//! listing.
//!
//! The line is not a resource: nothing is placed, nothing is undone, and `sync` does not try. The
//! four tests here are the four things that could go wrong with that claim — the promise is not
//! reachable from the files, a copy is not what the report says, `sync` acts on it anyway, or a
//! half-written line is accepted and means something nobody wrote.
//!
//! **`restic` stands in on `PATH`, as the scheduler stub does next door.** The program really
//! runs, really parses the JSON the report reads, and never reaches a repository.

use crate::harness::Fixture;

const LINE: &str = "invariant:seforim @path=/srv/seforim,backed-by=restic:b2:seforim,max-age=7d\n";

/// A `restic` that prints `body` for `snapshots --json`, and records that it was asked at all.
fn stub_restic(f: &Fixture, body: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let stub = f.root.join("stub-bin");
    std::fs::create_dir_all(&stub).unwrap();
    let script = stub.join("restic");
    // The time the report will read, and a record of the call — the second half is what lets the
    // `sync` test prove the promise was not converged.
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\necho \"$@\" >> '{}'\nprintf '%s' '{body}'\n",
            f.root.join("restic.log").display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    stub
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(unix)]
#[test]
fn a_copy_younger_than_the_promise_is_reported_as_holding() {
    let f = Fixture::new("74-fresh");
    f.write_module(LINE);
    let stub = stub_restic(&f, &format!(r#"[{{"time":"{}"}}]"#, now()));

    let (out, code) = f.run_with_path(&stub, &["check", "durability"]);
    assert_eq!(code, 0, "a fresh copy was not accepted:\n{out}");
    assert!(
        out.contains("1 invariant(s) hold"),
        "the report did not say the promise holds:\n{out}"
    );
}

#[cfg(unix)]
#[test]
fn an_old_copy_is_named_and_the_exit_code_says_so() {
    let f = Fixture::new("74-stale");
    f.write_module(LINE);
    let stub = stub_restic(&f, r#"[{"time":"2020-01-01T00:00:00Z"}]"#);

    let (out, code) = f.run_with_path(&stub, &["check", "durability"]);
    assert_eq!(code, 2, "a broken promise exited {code}:\n{out}");
    assert!(
        out.contains("the newest copy is"),
        "the report did not say how old the copy is:\n{out}"
    );
}

/// The line reaches `check` by being in the files and nowhere else — and `sync` leaves it alone.
///
/// **`sync` is the control that matters.** An invariant in `extras` would be dispatched by a
/// phase, and the phase that owns a backup is not one Shall has: a 3am `restic backup` is a
/// decision, and an unreachable repository would redden every sync on the machine.
#[cfg(unix)]
#[test]
fn a_sync_neither_installs_nor_backs_anything_up_for_a_promise() {
    let f = Fixture::new("74-sync-inert");
    f.write_module(LINE);
    let stub = stub_restic(&f, &format!(r#"[{{"time":"{}"}}]"#, now()));

    let (out, code) = f.run_with_path(&stub, &["-y", "sync", "--dry-run"]);
    assert_eq!(
        code, 0,
        "a config holding one promise failed to sync:\n{out}"
    );
    assert!(
        !f.root.join("restic.log").exists(),
        "`sync` ran the backup tool for a promise:\n{out}"
    );
}

/// A line that names a tool but not a target says nothing checkable, and is refused at the file
/// naming the part it left out.
#[cfg(unix)]
#[test]
fn a_half_written_promise_is_refused_where_it_was_written() {
    let f = Fixture::new("74-half-line");
    f.write_module("invariant:seforim @path=/srv/seforim,backed-by=restic,max-age=7d\n");

    let (out, code) = f.run(&["check", "durability"]);
    assert_eq!(code, 1, "a `backed-by` with no target was accepted:\n{out}");
    assert!(
        out.contains("@backed-by"),
        "the refusal does not name the option that is wrong:\n{out}"
    );
}

/// `~` is a path a person writes and a backup tool cannot read: `restic --path '~/Documents'`
/// looks for a directory literally called `~`. It is expanded through the same account helper
/// `link:` uses, so the two agree about what `~` means — and no `~` reaches the tool.
#[cfg(unix)]
#[test]
fn a_tilde_path_is_expanded_before_the_tool_is_asked() {
    let f = Fixture::new("74-tilde");
    f.write_module(
        "invariant:seforim @path=~/Documents/seforim,backed-by=restic:b2:seforim,max-age=7d\n",
    );
    let stub = stub_restic(&f, &format!(r#"[{{"time":"{}"}}]"#, now()));

    let (out, code) = f.run_with_path(&stub, &["check", "durability"]);
    assert_eq!(code, 0, "a `~` path was not accepted:\n{out}");
    let asked = std::fs::read_to_string(f.root.join("restic.log")).expect("the tool was asked");
    assert!(
        !asked.contains('~'),
        "the tool was asked about a literal `~`, which it cannot expand: {asked}"
    );
    assert!(
        asked.contains("/Documents/seforim"),
        "the path lost its tail on the way to the tool: {asked}"
    );
}
