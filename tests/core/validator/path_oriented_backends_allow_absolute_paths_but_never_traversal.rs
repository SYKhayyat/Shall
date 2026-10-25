use super::*;

#[test]
fn path_oriented_backends_allow_absolute_paths_but_never_traversal() {
    // `link` legitimately names a filesystem path — an absolute path is allowed.
    assert!(Validator::validate_package_name_for("/home/me/.vimrc", "link").is_ok());
    assert!(Validator::validate_package_name_for("/tmp/shall-link-src", "link").is_ok());
    // …but `..` traversal is STILL blocked, for every backend including path-oriented ones.
    assert!(Validator::validate_package_name_for("/home/../etc/shadow", "link").is_err());
    assert!(Validator::validate_package_name_for("../secrets", "link").is_err());
    // …and shell-injection characters are still blocked (backslash, $(), etc.).
    assert!(Validator::validate_package_name_for("/tmp/$(rm -rf)", "link").is_err());
}
