use super::*;

#[test]
fn strict_validation_blocks_absolute_paths_for_normal_backends() {
    // A leading slash on an ordinary package name is an injection attempt.
    assert!(Validator::validate_package_name("/etc/passwd").is_err());
    assert!(Validator::validate_package_name_for("/etc/passwd", "apt").is_err());
    assert!(Validator::validate_package_name_for("ripgrep", "apt").is_ok());
    // github owner/repo and web URLs (no leading slash) pass either way.
    assert!(Validator::validate_package_name_for("BurntSushi/ripgrep", "github").is_ok());
}
