use super::*;

#[test]
fn extract_targets_drops_binary_flags_and_keyword() {
    assert_eq!(
        extract_targets(&s(&["apt", "install", "-y", "curl", "htop"])),
        vec!["curl", "htop"]
    );
    assert_eq!(
        extract_targets(&s(&["pacman", "-S", "ripgrep"])),
        vec!["ripgrep"]
    );
    assert_eq!(
        extract_targets(&s(&["dnf", "remove", "--assumeyes", "nano"])),
        vec!["nano"]
    );
}
