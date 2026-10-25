use super::*;

#[test]
fn it_strips_escapes_and_collapses_crlf() {
    let input = "\u{1b}[32mSuccessfully installed\u{1b}[0m package-1.2.3\r\n";
    assert_eq!(sanitize(input), "Successfully installed package-1.2.3");
}
