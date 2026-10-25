use super::*;

#[test]
fn detect_operation_long_form() {
    assert_eq!(
        detect_operation(&s(&["apt", "install", "curl"])),
        Some(HookOp::Install)
    );
    assert_eq!(
        detect_operation(&s(&["dnf", "remove", "nano"])),
        Some(HookOp::Remove)
    );
    assert_eq!(
        detect_operation(&s(&["apt-get", "purge", "x"])),
        Some(HookOp::Remove)
    );
    assert_eq!(
        detect_operation(&s(&["apk", "add", "jq"])),
        Some(HookOp::Install)
    );
    assert_eq!(
        detect_operation(&s(&["brew", "uninstall", "wget"])),
        Some(HookOp::Remove)
    );
}
