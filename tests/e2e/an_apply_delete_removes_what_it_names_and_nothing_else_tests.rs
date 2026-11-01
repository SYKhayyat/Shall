//! `apply X --delete` removes X, from the files and the machine, and nothing else (`C7`,
//! rule II.70).
//!
//! The twin of `an_apply_makes_what_it_names_true_and_nothing_else_tests.rs`, and it needs its
//! own file rather than a test in that one: the failure mode is the opposite direction. A run
//! that converges the whole config *installs* the rest; a removal that does makes every removal
//! a potential mass removal, on a machine the user has just asked to take one thing off.
//!
//! `dir:` again — no manager, no root, no network. Whether the directory is still there is the
//! assertion.

use crate::harness::{decl, Fixture};
use std::path::Path;

fn line(path: &Path) -> String {
    format!("dir:{}", decl(path))
}

fn apply(f: &Fixture, path: &Path) {
    let spec = line(path);
    let (out, code) = f.run(&["-y", "apply", spec.as_str()]);
    assert_eq!(
        code,
        0,
        "the fixture's own apply of {} failed:\n{out}",
        path.display()
    );
    assert!(path.is_dir(), "the fixture's apply created nothing:\n{out}");
}

fn module(f: &Fixture) -> String {
    std::fs::read_to_string(f.cfg().join("modules/imperative.txt")).unwrap_or_default()
}

/// Two declarations, both applied. One is deleted; the other stays declared and stays on the
/// machine.
#[test]
fn a_delete_removes_its_line_and_its_resource_and_leaves_the_rest() {
    let f = Fixture::new("c7-delete-names");
    let keep = f.root.join("keep-dir");
    let drop = f.root.join("drop-dir");
    apply(&f, &keep);
    apply(&f, &drop);

    let spec = line(&drop);
    let (out, code) = f.run(&["-y", "apply", "--delete", spec.as_str()]);
    assert_eq!(code, 0, "`apply --delete` failed:\n{out}");
    assert!(!drop.is_dir(), "`apply --delete` removed nothing:\n{out}");
    assert!(
        keep.is_dir(),
        "`apply --delete` removed `keep-dir`, which it was not asked about:\n{out}"
    );

    let module = module(&f);
    assert!(
        module.contains(&line(&keep)),
        "`apply --delete` took away a declaration it was not asked about:\n{module}"
    );
    assert!(
        !module.contains(&line(&drop)),
        "`apply --delete` left the line it was asked to remove:\n{module}"
    );
}

/// A managed package whose line is already gone is drift — and a narrowed deletion must not
/// reap it while removing something else. One removal command, one removal.
#[test]
fn a_delete_does_not_reap_the_rest_of_the_machine() {
    let f = Fixture::new("c7-delete-no-reap");
    let drift = f.root.join("drift-dir");
    let drop = f.root.join("drop-dir");
    apply(&f, &drift);
    apply(&f, &drop);

    // `drift` loses its line: installed, managed, declared nowhere.
    f.write("modules/imperative.txt", &format!("{}\n", line(&drop)));

    let spec = line(&drop);
    let (out, code) = f.run(&["-y", "apply", "--delete", spec.as_str()]);
    assert_eq!(code, 0, "`apply --delete` failed:\n{out}");
    assert!(!drop.is_dir(), "`apply --delete` removed nothing:\n{out}");
    assert!(
        drift.is_dir(),
        "`apply --delete` reaped a second package on its way out. Removing drift is what a \
         whole-machine `sync` does (II.70):\n{out}"
    );
}

/// `--absent` is a package idea: it keeps a *name* off, because a manager can put the name back
/// from a module the user forgot. A resource line has no such second source — taking the line
/// out is the whole of it — so the flag is refused rather than written as a line nothing reads.
#[test]
fn absent_is_about_packages_so_a_resource_line_is_refused() {
    let f = Fixture::new("c7-delete-absent-refused");
    let drop = f.root.join("drop-dir");
    apply(&f, &drop);

    let spec = line(&drop);
    let (out, code) = f.run(&["-y", "apply", "--delete", spec.as_str(), "--absent"]);
    assert_ne!(code, 0, "`--absent` on a resource line exited 0:\n{out}");
    assert!(
        out.contains("resource"),
        "the refusal does not say why an `absent:` line is the wrong thing here:\n{out}"
    );
    assert!(
        drop.is_dir(),
        "the refusal still removed the resource it refused to declare absent:\n{out}"
    );
    assert!(
        module(&f).contains(&line(&drop)),
        "the refusal still took the line out of the files:\n{}",
        module(&f)
    );
}
