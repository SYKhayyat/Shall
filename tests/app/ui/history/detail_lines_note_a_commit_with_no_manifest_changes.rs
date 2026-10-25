use super::*;

#[test]
fn detail_lines_note_a_commit_with_no_manifest_changes() {
    let c = cv("a1b2c3d", "docs only", &[]);
    assert!(detail_lines(&c)
        .iter()
        .any(|l| l.contains("No manifest changes")));
}
