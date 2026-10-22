use super::*;

#[test]
fn a_link_with_a_windows_path_is_a_link_not_an_expression() {
    // II.2 vs II.4: `looks_like_expression` fires on `\`, and a Windows path is full of
    // them. The typed prefix has to win, or `link:C:\Users\me\.vimrc` parses as set math.
    let stmt = parse_line(r"link:C:\Users\me\.vimrc@target=~/.vimrc").unwrap();
    assert!(matches!(stmt, Statement::Link(..)), "got {:?}", stmt);
    // And an actual expression with no statement prefix still reads as one.
    assert!(matches!(
        parse_line("editors | fonts").unwrap(),
        Statement::Expr(_)
    ));
}
