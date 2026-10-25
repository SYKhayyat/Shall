use super::*;

#[test]
fn pip_is_last_because_it_can_break_your_system_python() {
    let out = starter_order(&["pip".into(), "cargo".into(), "apt".into()]);
    assert_eq!(out, ["apt", "cargo", "pip"]);
}
