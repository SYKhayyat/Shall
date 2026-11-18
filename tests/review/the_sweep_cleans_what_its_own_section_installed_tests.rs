//! The sweep cleans up what its own section installed, and it actually runs.
//!
//! **A cleanup that is defined and never called is the defect it was written for.** The sweep
//! installs a canary, its manager's uninstall keeps the shim, and the section passes because
//! `removal_leaves_binary` said the product was honest about it — after which the file stays on
//! the host. The next section installs the same canary, finds the name resolving exactly where it
//! resolved before, and has no evidence left: Shall's warning about a bin directory only exists
//! when that directory is *off* `PATH`, and a leftover shim that resolves is by definition in one
//! that is on it. Measured on one Windows host in two consecutive runs, `pipx: pycowsay` and then
//! `nimble: nimjson` (#197).
//!
//! `scripts/harness-logic-test.sh` pins what `clear_own_leftover` *does* — it takes the
//! resolution read before the section's install as its only authority, removes the file when that
//! was empty, and leaves it alone when it was not. It cannot pin that the sweep *calls* it,
//! because the call sites are inside `lifecycle()`, which is longer than the lifter's runaway
//! guard allows. So this file does, and it is the half that was actually missing: the function is
//! four lines of `rm` and the bug is a section that never runs it.

use std::path::PathBuf;

fn harness() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/integration-windows.sh");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn the_sweep_calls_its_own_leftover_cleanup_after_each_removal_path() {
    let src = harness();
    assert!(
        src.contains("clear_own_leftover() {"),
        "the sweep must keep a function that removes what its own section left behind: the \
         manager keeps its shim, the next section cannot prove delivery, and the sweep stops \
         being runnable twice on one host"
    );

    // Both ways a lifecycle can end without the removal having been observed: the manager that
    // refuses to remove at all, and the ordinary path. A section that returns through either one
    // leaves its shim behind, so both are call sites.
    let call = "clear_own_leftover \"$cbin\" \"$_prepath\"";
    let calls = src.matches(call).count();
    assert!(
        calls >= 2,
        "the sweep calls clear_own_leftover {calls} time(s); it has to be on both removal paths \
         (the unsupported refusal and the ordinary removal), or one of them leaves a shim that \
         makes the next run of this sweep fail for a reason the product did not cause"
    );

    // The one argument that must not be dropped: the cleanup's authority is the resolution read
    // BEFORE this section installed anything. Without it, another manager's file is a candidate
    // for deletion.
    assert!(
        src.contains("_prepath=\"$(path_of \"$cbin\")\""),
        "the sweep must read the prior resolution before the install, because that is the only \
         thing that says whose file a leftover is"
    );
}
