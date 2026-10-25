use super::*;

/// The family, not the finding. Every ecosystem that installs into `$HOME` and leaves the
/// PATH to you must answer, or it is the next `pub`.
#[tokio::test]
async fn every_per_user_ecosystem_answers() {
    let (exec, cfg) = mocked(&[]);
    for be in [
        "pub", "nimble", "go", "cargo", "gem", "luarocks", "mix", "stack", "krew", "pipx",
        "composer",
    ] {
        assert!(
            !user_bin_dirs(be, &cfg, &exec).await.is_empty(),
            "{be} installs into a user directory and this table does not know where"
        );
    }
}
