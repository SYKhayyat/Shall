use super::*;

/// `~/bin/tool` expands to the home directory, because the availability check does not.
#[test]
fn a_leading_tilde_expands_to_home() {
    let expanded = expand_binary("~/bin/tool");
    assert!(!expanded.starts_with('~'), "{}", expanded);
    assert!(
        expanded.ends_with("bin/tool") || expanded.ends_with("bin\\tool"),
        "{}",
        expanded
    );
    // A bare command and a `~` that is not a leading path segment are left untouched.
    assert_eq!(expand_binary("ufw"), "ufw");
    assert_eq!(expand_binary("/opt/x~y"), "/opt/x~y");
}
