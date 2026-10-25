use super::*;

#[test]
fn detect_operation_pacman_short_flags() {
    assert_eq!(
        detect_operation(&s(&["pacman", "-S", "htop"])),
        Some(HookOp::Install)
    );
    assert_eq!(
        detect_operation(&s(&["pacman", "-Rns", "htop"])),
        Some(HookOp::Remove)
    );
    assert_eq!(
        detect_operation(&s(&["pacman", "-Syu"])),
        Some(HookOp::Install)
    );
}
