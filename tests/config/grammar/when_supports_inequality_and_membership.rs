use super::*;

#[test]
fn when_supports_inequality_and_membership() {
    assert_eq!(stmts("when os != windows {\n  apt:htop\n}\n").len(), 1);
    assert_eq!(
        stmts("when arch in [x86_64, aarch64] {\n  apt:htop\n}\n").len(),
        1
    );
    assert!(stmts("when arch in [aarch64] {\n  apt:htop\n}\n").is_empty());
}
