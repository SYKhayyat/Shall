use super::*;

#[test]
fn stripping_the_marker_does_not_move_the_lines_under_it() {
    // An error that names the wrong line is how a one-line offset survives review.
    let (_, body) = Dialect::of("#rhai\nlet a = 1;\nlet b = 2;\n");
    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(lines[0], "", "line 1 should be blank, not gone");
    assert_eq!(lines[1], "let a = 1;");
    assert_eq!(lines[2], "let b = 2;");
}
