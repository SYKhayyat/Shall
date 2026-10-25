use super::*;

/// The rehearsal asks whether the config RESOLVES, so it runs `eval` — not `check`, which
/// also exits 2 for "nothing is installed here" and made `try` unable to reject anything.
#[test]
fn the_rehearsal_asks_whether_the_config_resolves() {
    let cmd = argv("docker", "i", "/cfg");
    assert_eq!(cmd.last().map(String::as_str), Some("eval"));
    assert!(!cmd.iter().any(|a| a == "check"), "{:?}", cmd);
}
