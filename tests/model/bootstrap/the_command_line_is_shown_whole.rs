use super::*;

/// The confirmation shows the command in full — that is the entire value of asking.
#[test]
fn the_command_line_is_shown_whole() {
    let r = row("brew", None);
    let line = r.command_line();
    assert!(line.contains("/bin/sh"), "{}", line);
    assert!(line.contains("install brew"), "{}", line);
}
