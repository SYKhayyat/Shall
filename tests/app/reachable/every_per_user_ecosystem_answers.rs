use super::*;
use shall::core::executor::DryRunOutput;

/// The family, not the finding. Every ecosystem that installs into `$HOME` and leaves the
/// PATH to you must answer, or it is the next `pub`.
#[tokio::test]
async fn every_per_user_ecosystem_answers() {
    // `go env GOPATH`, stated rather than fallen through (`#162`): the answer is empty, and this
    // test asserts that even an empty answer still names a directory rather than nothing.
    let (exec, cfg) = mocked(&[("go env GOPATH", "")]);
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
