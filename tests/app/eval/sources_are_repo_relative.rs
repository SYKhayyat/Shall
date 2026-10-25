use super::*;

/// Sources are reported relative to the repo — the difference that would otherwise make
/// every line of a cross-machine diff differ for no reason the configuration is
/// responsible for.
#[test]
fn sources_are_repo_relative() {
    assert_eq!(
        repo_relative(
            "/home/a/.config/shall/modules/x.txt:7",
            Path::new("/home/a/.config/shall")
        ),
        "modules/x.txt:7"
    );
    // A file-level origin has no line, and must not grow one.
    assert_eq!(repo_relative("/repo/active", Path::new("/repo")), "active");
}
