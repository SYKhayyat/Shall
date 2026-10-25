use super::*;

#[test]
fn parse_manifest_changes_keeps_content_lines_drops_headers() {
    // A realistic `git show` diff body: file headers, hunk header, context, +/- lines.
    let diff = "diff --git a/modules/dev.txt b/modules/dev.txt\n\
                index 111..222 100644\n\
                --- a/modules/dev.txt\n\
                +++ b/modules/dev.txt\n\
                @@ -1,3 +1,3 @@\n\
                \x20apt:curl\n\
                -apt:nano\n\
                +cargo:ripgrep\n\
                +# a comment line, not a package\n";
    let changes = parse_manifest_changes(diff);
    // Kept: the real +/- package lines. Dropped: ---/+++ headers, context, and the comment.
    assert_eq!(
        changes,
        vec!["- apt:nano".to_string(), "+ cargo:ripgrep".to_string()]
    );
}
