use super::*;

/// The Unix entries keep working, and an ordinary system file that is not on the list is
/// allowed through — the control that keeps this from becoming "refuse everything".
#[test]
fn unix_entries_are_still_enforced_and_innocent_paths_still_pass() {
    assert!(Validator::refuse_forbidden(Path::new("/etc/shadow")).is_err());
    // Prefix matches stay on whole components: a child of a forbidden name is caught,
    // a sibling merely sharing letters is not.
    assert!(Validator::refuse_forbidden(Path::new("/etc/shadow/backup")).is_err());
    assert!(Validator::refuse_forbidden(Path::new("/etc/shadowlocks/old")).is_ok());
    assert!(Validator::refuse_forbidden(Path::new("/etc/nginx/nginx.conf")).is_ok());
}
