use super::*;

#[test]
fn detail_lines_show_metadata_and_changes() {
    let c = cv(
        "a1b2c3d",
        "swap nano for ripgrep",
        &["+ cargo:rg", "- apt:nano"],
    );
    let joined = detail_lines(&c).join("\n");
    assert!(joined.contains("Commit  : a1b2c3d"));
    assert!(joined.contains("Message : swap nano for ripgrep"));
    assert!(joined.contains("Manifest changes in this commit:"));
    assert!(joined.contains("+ cargo:rg"));
    assert!(joined.contains("- apt:nano"));
}
