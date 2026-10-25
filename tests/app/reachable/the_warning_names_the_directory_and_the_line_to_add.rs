use super::*;

/// The message has to carry the fix, not the diagnosis. A warning naming neither the
/// directory nor the line to add is one more thing to look up.
#[tokio::test]
async fn the_warning_names_the_directory_and_the_line_to_add() {
    let (exec, cfg) = mocked(&[]);
    // Pick whichever of these is genuinely unreachable on the machine running the test;
    // asserting on a fixed one makes this pass or fail on the host's PATH rather than on
    // the code.
    let mut found = None;
    for b in ["pub", "composer", "mix", "krew", "luarocks", "nimble"] {
        if let Some(m) = unreachable_warning(b, &cfg, &exec).await {
            found = Some((b, m));
            break;
        }
    }
    let Some((be, msg)) = found else {
        // Every one of them is on PATH here. Nothing to assert, and nothing wrong.
        return;
    };
    let dir = conventional_bin_dir(be).unwrap();
    // The instruction has to be runnable on THIS platform: `export PATH=` in a cmd window
    // is a wrong answer, not a terse one.
    assert!(
        msg.contains(if cfg!(windows) {
            "powershell -NoProfile -Command"
        } else {
            "export PATH="
        }),
        "the fix is not in this platform's syntax: {msg}"
    );
    assert!(
        msg.contains(&dir.display().to_string()),
        "no directory: {msg}"
    );
    assert!(msg.contains(be), "does not say which manager: {msg}");
}
