use super::*;

#[test]
fn a_user_made_snapshot_is_not_owned_and_is_left_alone() {
    assert!(!snap("12", "Windows Update", "windows_restore").is_shall_owned());
    assert!(!snap("tank/root@weekly", "manual weekly", "zfs").is_shall_owned());
    // The word appears inside the user's own naming: a substring test claims these and
    // `prune_with_policy` deletes them.
    assert!(!snap("tank/home@marshall_weekly", "Marshall weekly", "zfs").is_shall_owned());
    assert!(!snap("13", "Shallots: planting notes", "windows_restore").is_shall_owned());
}
