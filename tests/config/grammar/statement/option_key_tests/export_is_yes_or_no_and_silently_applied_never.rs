use super::*;

/// II.94's opt-out: `@export=no` keeps a line out of `shall export` while `sync` still applies
/// it. Two words, closed — a typo must not read as the default, because an opt-out that silently
/// exports the line is the thing it exists to prevent.
#[test]
fn a_no_marks_the_line_opt_out_and_a_yes_does_not() {
    use shall::config::grammar::statement::Statement;
    let line = parse_line("link:./vimrc @target=~/.vimrc,export=no").unwrap();
    assert!(
        matches!(line, Statement::Link(_, ref o) if o.one("export") == Some("no"))
            && !line.exports(),
        "{line:?}"
    );

    let yes = parse_line("link:./vimrc @target=~/.vimrc,export=yes").unwrap();
    assert!(
        yes.exports(),
        "an explicit `@export=yes` opted the line out: {yes:?}"
    );
}

#[test]
fn a_third_word_is_refused_naming_the_two() {
    let err = parse_line("link:./vimrc @target=~/.vimrc,export=nope").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("@export=nope"), "did not name the word: {msg}");
    assert!(
        msg.contains("yes") && msg.contains("no"),
        "did not name the vocabulary: {msg}"
    );
}

#[test]
fn a_package_line_takes_the_same_opt_out() {
    let no = parse_line("cargo:ripgrep @export=no").unwrap();
    assert!(!no.exports());
    let err = parse_line("cargo:ripgrep @export=maybe").unwrap_err();
    assert!(err.to_string().contains("is not a yes or a no"));
}
