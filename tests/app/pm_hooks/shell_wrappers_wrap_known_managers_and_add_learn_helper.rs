use super::*;

#[test]
fn shell_wrappers_wrap_known_managers_and_add_learn_helper() {
    let w = shell_wrappers("shall", "bash");
    assert!(w.contains("apt()"));
    assert!(w.contains("pacman()"));
    assert!(w.contains("command apt \"$@\""));
    assert!(w.contains("shalllearn()"));
    assert!(w.contains("hook-observe"));
}
