use super::*;

/// The class, not the character. `looks_like_expression` fires on four punctuation marks and
/// `\` is simply the one a real tool prints; a qualified line carrying any of them is either
/// a package or a refusal that names the line, and never profile algebra the user did not
/// write. This is here so the next punctuation class needs no third discovery.
#[test]
fn no_punctuation_turns_a_qualified_line_into_set_math() {
    for c in ['\\', '|', '&', '('] {
        let line = format!("winget:a{c}b");
        match parse_line(&line) {
            Ok(Statement::Package(_)) => {}
            Ok(Statement::Expr(e)) => panic!(
                "`{line}` was read as set math ({e}); the user is told a module cannot use \
                 a set expression, about a line that asked for none"
            ),
            Ok(other) => panic!("`{line}` parsed as {other:?}"),
            // A refusal is a fine answer — the name is the subject, and that is legible.
            Err(e) => assert!(
                e.what.contains("package name") || e.what.contains(&line),
                "`{line}` was refused without naming the name: {e}"
            ),
        }
    }
}
