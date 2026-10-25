use super::*;

#[test]
fn a_line_the_grammar_cannot_read_is_refused_before_the_file_is_touched() {
    // The root of the adopted.txt:69 wedge. `key_of` parsed and returned `None` on an
    // error, and the line was then appended anyway — so a write could put a parse error
    // into a file Shall generated, and every later command died reading it. Both live
    // sources produced real ones: `winget list`'s `ARP\Machine\X64\Android Studio`, and
    // the pm-hook taking its target off a `choco install "Google Chrome"` command line.
    let f = fx(&[("active", "Work\n"), ("profiles/Work", "use dev\n")]);
    for bad in [
        r"winget:ARP\Machine\X64\Android Studio",
        "choco:Google Chrome",
        "nosuchbackend:jq",
    ] {
        assert!(
            editor(&f).add(&Landing::Imperative.target(), bad).is_err(),
            "`{}` must be refused",
            bad
        );
    }
    assert_eq!(
        read(&f, "modules/imperative.txt"),
        "",
        "a refused write leaves no file behind"
    );
    // And the neighbouring statement kinds still write, so the check refuses only what
    // the grammar refuses.
    for good in ["apt:jq", "service:nginx@enabled=true", "cargo,apt:ripgrep"] {
        assert!(
            editor(&f).add(&Landing::Imperative.target(), good).is_ok(),
            "`{}` must still be accepted",
            good
        );
    }
}
