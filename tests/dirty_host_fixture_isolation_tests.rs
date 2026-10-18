//! The native sweep must not reuse another invocation's fixture state.
//!
//! This is a source-level control because running the native sweep requires Windows/macOS
//! managers. Its invariant is platform-independent: the run root is unique before any config,
//! data, ledger, or helper path is derived from `TMPDIR`.

use std::path::PathBuf;

fn script() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/integration-windows.sh");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn native_sweep_derives_all_fixtures_from_a_unique_run_root() {
    let body = script();
    let root = body
        .find("RUN_ROOT=\"$(mktemp -d ")
        .expect("the sweep must create a unique temporary root");
    let export = body
        .find("export TMPDIR=\"$RUN_ROOT\"")
        .expect("the sweep must scope TMPDIR to its unique root");
    let config = body
        .find("export SHALL_CONFIG_DIR=\"${TMPDIR}/shall-it-win-config\"")
        .expect("the config fixture must be derived from the run-scoped TMPDIR");
    let data = body
        .find("export SHALL_DATA_DIR=\"${TMPDIR}/shall-it-win-state\"")
        .expect("the data fixture must be derived from the run-scoped TMPDIR");
    let ledger = body
        .find("LEDGER=\"${TMPDIR}/shall-it-win-ledger\"")
        .expect("the ledger fixture must be derived from the run-scoped TMPDIR");

    assert!(root < export && export < config && config < data && data < ledger);
    assert!(
        body.contains("cleanup_run() { rm -rf \"$RUN_ROOT\"")
            && body.contains("trap cleanup_run EXIT INT TERM")
            && body.contains("cleanup_run' EXIT INT TERM")
            && body.contains("\ncleanup_run\n\n# --- Summary")
    );
}
