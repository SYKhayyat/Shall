use super::*;

/// Every string a writer in the tree stores in `ManagedPackage::source`, against the
/// sentence `why` gives back. One entry per writer, so a new writer with no arm here is
/// visible as a bare "recorded by" rather than as nothing at all.
#[test]
fn why_names_each_writer_by_the_string_that_writer_stores() {
    // `app/shell/mod.rs`, `app/leases.rs`, `app/snapshot_restore.rs`.
    assert_eq!(provenance("imperative"), "installed by `shall install`");
    // `app/adopt.rs`.
    assert_eq!(provenance("adopt"), "adopted from this machine");
    // `verbs/declare.rs` — `hook:<manager>`, one arm per manager it can be. Matching a
    // bare `hook` made this unreachable for every one of them.
    for manager in ["choco", "scoop", "apt", "brew"] {
        let said = provenance(&format!("hook:{manager}"));
        assert!(
            said.contains("caught by the hook") && said.contains(manager),
            "hook:{manager} answered {said:?}"
        );
    }
    // `verbs/declare.rs` local-file arm, `app/diagnostics.rs`, `verbs/plan.rs`,
    // `app/sync/mod.rs` — no dedicated sentence, and the fallback names them.
    for other in ["local-file", "diagnostics", "plan", "sync"] {
        assert_eq!(provenance(other), format!("recorded by {other}"));
    }
    // A declared package: `model/resolve.rs` stamps `file:line`, which is the answer
    // `why` exists to give.
    assert_eq!(
        provenance("modules/dev.txt:14"),
        "recorded by modules/dev.txt:14"
    );
}
