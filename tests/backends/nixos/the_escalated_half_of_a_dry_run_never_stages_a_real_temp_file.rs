use super::*;

#[tokio::test]
async fn the_escalated_half_of_a_dry_run_never_stages_a_real_temp_file() {
    // Parent does not exist, so place() takes the escalation branch: staging + root
    // install. Under a dry run neither may touch the machine.
    let dir = tempfile::tempdir().unwrap();
    let before: Vec<_> = std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();
    let dest = dir.path().join("not-here").join("shall-packages.nix");
    let (core, _vfs, mock) = dry_core();

    core.place(&dest, "BODY")
        .await
        .expect("records the escalation");
    assert!(!dest.exists());

    let after: Vec<_> = std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();
    let new_files: Vec<_> = after
        .iter()
        .filter(|n| !before.contains(n))
        // Files only: parallel tests create and remove *directories* under shared
        // `temp_dir` (the listing-cache fixture among them), and one caught mid-cleanup
        // by this snapshot read exactly like a staged file.
        .filter(|n| n.to_string_lossy().starts_with("shall-"))
        .filter(|n| {
            std::fs::metadata(std::env::temp_dir().join(n))
                .map(|m| m.is_file())
                .unwrap_or(false)
        })
        .collect();
    assert!(
        new_files.is_empty(),
        "a dry run staged real temp files: {new_files:?}"
    );
    assert!(
        mock.get_calls()
            .await
            .iter()
            // Substring, not prefix: on Unix an escalated run records
            // `sudo -n install -m 0644 …`, and a prefix match would make this test
            // pass on Windows and root only.
            .any(|c| c.contains("install -m 0644")),
        "the install step is still previewed in the run log"
    );
}
