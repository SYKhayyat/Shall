use super::*;

/// **The half that stops this being noise on every failure.** A mirror that was down, a
/// name no repository carries, a disk that filled — none of those are the lockfile's doing,
/// and advice to unpin something would send the reader after the wrong thing entirely.
#[test]
fn a_failure_that_does_not_quote_the_pin_gets_no_advice() {
    let (_dir, config) = config_with(r#"{"locks":{"apt:libudev1":"255.4-1ubuntu8.17"}}"#);
    for unrelated in [
        "E: Failed to fetch http://archive.ubuntu.com — Connection timed out",
        "E: Unable to locate package libudev1",
        "dpkg: error: failed to write: No space left on device",
    ] {
        assert!(
            on_install_failure(&config, "apt", "libudev1", unrelated).is_none(),
            "advice was offered for a failure the pin did not cause: {unrelated}"
        );
    }
}
