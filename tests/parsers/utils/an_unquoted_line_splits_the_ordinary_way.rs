use super::*;

/// A line with nothing quoted must split exactly as whitespace splitting would, or the
/// `quoted = true` option would be a behaviour change for every row that set it.
#[test]
fn an_unquoted_line_splits_the_ordinary_way() {
    for line in ["git 2.40.0 installed", "  ripgrep\t13.0.0  ", "solo"] {
        let mine: Vec<String> = split_columns(line);
        let theirs: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(mine, theirs, "{line:?}");
    }
}
