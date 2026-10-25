use super::*;

/// **A dry run edits boot configuration only in its own imagination.** `place` used to
/// write with raw `std::fs` whenever the parent was writable — which on a scratch config
/// dir is always — so `--dry-run` wrote `/etc/nixos` for real.
#[tokio::test]
async fn a_dry_run_records_the_generated_file_instead_of_writing_it() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("shall-packages.nix");
    let (core, vfs, _mock) = dry_core();

    core.place(&dest, "BODY").await.expect("places");

    assert!(!dest.exists(), "a dry run must not write the file for real");
    assert_eq!(
        vfs.get(&dest).map(|v| v.value().clone()).as_deref(),
        Some("BODY"),
        "the dry run records what would have landed"
    );
}
