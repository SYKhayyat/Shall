use super::*;

#[test]
fn nothing_upgradable_is_nothing() {
    assert!(parse_pacman_outdated("").is_empty());
    // `pacman -Qu` prints nothing at all when everything is current.
    assert!(parse_pacman_outdated("\n\n").is_empty());
}
