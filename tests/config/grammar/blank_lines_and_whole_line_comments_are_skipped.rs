use super::*;

#[test]
fn blank_lines_and_whole_line_comments_are_skipped() {
    let out = stmts("# a comment\n\n   \napt:curl\n");
    assert_eq!(out.len(), 1);
}
