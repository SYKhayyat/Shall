//! `apply X` makes X true and nothing else (`C7`, rule II.70).
//!
//! The measured cost of the old shape: the 2026-10-26 native sweeps' own `adopt` had put 43
//! packages (macOS) and 103 (Windows) into the model, and `install` was "edit a line, then
//! converge the whole configuration" — so `shall -y apply cargo:hexyl` ran a plan whose one
//! change was `brew:rustup`, and the harness that read exit 0 as "installed for real" then
//! failed its listing check against a crate nobody had installed (#112).
//!
//! **`dir:` is the machine here.** No root, no network, no manager: whether a directory exists
//! is the entire assertion, and the "package" is the path. `dir:` is also what proves `apply`
//! reaches the non-package half at all — the thing `install` never owned and `apply` does.

use crate::harness::{decl, Fixture};
use std::path::Path;

fn line(path: &Path) -> String {
    format!("dir:{}", decl(path))
}

fn apply(f: &Fixture, path: &Path) -> String {
    let spec = line(path);
    let (out, code) = f.run(&["-y", "apply", spec.as_str()]);
    assert_eq!(
        code,
        0,
        "the fixture's own apply of {} failed:\n{out}",
        path.display()
    );
    out
}

fn module(f: &Fixture) -> String {
    std::fs::read_to_string(f.cfg().join("modules/imperative.txt")).unwrap_or_default()
}

/// One pending declaration in the config, one named on the command line.
#[test]
fn an_apply_makes_its_line_true_and_leaves_the_rest_of_the_config_pending() {
    let f = Fixture::new("c7-apply-names");
    let pending = f.root.join("pending-dir");
    let named = f.root.join("named-dir");
    f.write_module(&format!("{}\n", line(&pending)));

    let out = apply(&f, &named);
    assert!(
        named.is_dir(),
        "`apply` did not make the declaration it named true:\n{out}"
    );
    assert!(
        !pending.is_dir(),
        "`apply` converged the whole configuration: the pending `dir:` was created too. The \
         whole-machine converge is `sync`'s (II.70):\n{out}"
    );
    assert!(
        out.contains("shall sync"),
        "the rest of the config is pending after a narrowed run, and one line has to say \
         so — naming the command that converges it:\n{out}"
    );
}

/// The line is recorded, which is the half that makes `apply` a declaration rather than a
/// side effect: a later whole-machine `sync` must keep what was applied, not remove it.
#[test]
fn an_apply_records_the_line_so_a_sync_keeps_it() {
    let f = Fixture::new("c7-apply-records");
    let named = f.root.join("named-dir");

    apply(&f, &named);
    assert!(
        module(&f).contains(&line(&named)),
        "`apply` applied the declaration and wrote no line for it:\n{}",
        module(&f)
    );

    let (out, code) = f.run(&["-y", "sync"]);
    assert_eq!(code, 0, "the follow-up `sync` failed:\n{out}");
    assert!(
        named.is_dir(),
        "`sync` removed a declaration `apply` had just written:\n{out}"
    );
}

/// A package Shall manages whose line was taken away by hand is drift — and drift is `sync`'s
/// finding. An `apply` that reaped it would make applying anything a mass removal.
#[test]
fn an_apply_does_not_reap_a_package_whose_declaration_is_gone() {
    let f = Fixture::new("c7-apply-no-reap");
    let drift = f.root.join("drift-dir");
    let named = f.root.join("second-dir");

    apply(&f, &drift);
    assert!(drift.is_dir(), "the fixture's apply created nothing");

    // The line goes by hand. The package is now installed, managed, and declared nowhere.
    f.write("modules/imperative.txt", "");

    let out = apply(&f, &named);
    assert!(
        named.is_dir(),
        "the named declaration was not applied:\n{out}"
    );
    assert!(
        drift.is_dir(),
        "`apply` removed a package nothing declares. Reaping drift is a whole-machine \
         converge, and a narrowed run does none (II.70):\n{out}"
    );
}

/// S15 stands: the write comes first, so a line that cannot be written applies nothing —
/// including nothing else the config asks for.
#[test]
fn an_apply_whose_line_cannot_be_read_applies_nothing() {
    let f = Fixture::new("c7-apply-refused-write");
    let pending = f.root.join("pending-dir");
    f.write_module(&format!("{}\n", line(&pending)));

    let (out, code) = f.run(&["-y", "apply", "nosuchbackend:foo"]);
    assert_ne!(code, 0, "the refused line exited 0:\n{out}");
    assert!(
        !pending.is_dir(),
        "a refused line still converged the config — the line has to be accepted before \
         anything is applied (S15):\n{out}"
    );
}

/// A failure inside a run is about what was asked for, and it says which declaration. The Q34
/// apology machinery is gone with the converge that made
/// it necessary.
#[test]
fn a_failed_apply_names_its_declaration_and_applies_nothing_else() {
    let f = Fixture::new("c7-apply-failure");
    let pending = f.root.join("pending-dir");
    f.write_module(&format!("{}\n", line(&pending)));

    // `web` has to be a manager this host uses, or the declaration is refused at the grammar
    // before the download is ever attempted — a different test than this one.
    let priority = std::fs::read_to_string(f.cfg().join("priority")).unwrap_or_default();
    if !priority.lines().any(|l| l.trim() == "web") {
        f.write("priority", &format!("{priority}web\n"));
    }

    // The house's hermetic failure: port 9 is `discard`, closed everywhere.
    let spec = "web:http://127.0.0.1:9/shall-no-such-file.bin@unverified";
    let (out, code) = f.run(&["-y", "apply", spec]);
    assert_ne!(code, 0, "a failed apply exited 0:\n{out}");
    assert!(
        out.contains("shall-no-such-file.bin"),
        "the failure does not name the declaration it was about:\n{out}"
    );
    assert!(
        !pending.is_dir(),
        "a failed apply converged the rest of the config:\n{out}"
    );
}
