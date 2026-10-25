use super::*;

/// The directory Shall deploys into is a user directory like any other, and it is the one
/// Shall is answerable for. CI, 2026-07-29: `github installed sharkdp/fd for real`,
/// `github: list shows fd`, `github: fd is on PATH` FAILED — `~/.local/bin` is not on a
/// clean Windows runner's PATH and nothing said so.
///
/// Asserted against the *config's* directory, not merely "some directory": the three
/// backends built `~/.local/bin` from `dirs::home_dir()` themselves until this change, so a
/// warning could name a directory the deploy never used, and a sandboxed run deployed into
/// the developer's real home.
#[tokio::test]
async fn the_directory_shall_itself_deploys_into_answers() {
    let (exec, cfg) = mocked(&[]);
    for be in ["github", "web", "appimage", "shim"] {
        assert_eq!(
            user_bin_dirs(be, &cfg, &exec).await,
            vec![cfg.bin_dir.clone()],
            "{be} deploys through Shall's own bin dir and this does not say so"
        );
    }
}
