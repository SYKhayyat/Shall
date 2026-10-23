use super::*;

/// Measured in a `homebrew/brew` container: brew prints `==> Auto-updating Homebrew...`
/// and its env hints ahead of the payload. Parsing the whole output fails, and a failed
/// parse here reports nothing outdated on exactly the machines that had not run brew in a
/// while.
#[test]
fn a_banner_before_the_json_does_not_hide_the_answer() {
    let noisy = format!(
        "==> Auto-updating Homebrew...\nAdjust how often this is run with $HOMEBREW…\n{}",
        BREW
    );
    assert_eq!(parse_brew_outdated(&noisy).len(), 3);
}
