use super::*;

/// The three copies of the prefix list had drifted, and this is the line that proved it:
/// `starts_with_statement_prefix` had never heard of `setting:`, so a Windows registry key
/// — full of backslashes — was handed to the set-expression parser instead of the setting
/// parser. `generate:` had the same hole.
#[test]
fn a_prefix_whose_payload_looks_like_set_math_is_still_that_statement() {
    for line in [
        r"generate:C:\tools\list-packages.ps1",
        r"link:C:\Users\me\.vimrc @target=~/.vimrc",
    ] {
        match p(line) {
            Ok(Statement::Expr(e)) => panic!("`{line}` was read as a set expression: {e}"),
            Ok(_) => {}
            Err(e) => panic!("`{line}` did not parse: {e}"),
        }
    }
}
