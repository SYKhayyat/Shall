use super::*;

#[test]
fn detect_operation_ignores_read_only() {
    assert_eq!(detect_operation(&s(&["apt", "list", "--installed"])), None);
    assert_eq!(detect_operation(&s(&["pacman", "-Q"])), None);
    assert_eq!(detect_operation(&s(&["dnf", "search", "editor"])), None);
}
