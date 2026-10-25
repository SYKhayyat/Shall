use super::*;

#[test]
fn hook_op_round_trips() {
    assert_eq!(HookOp::parse("install"), Some(HookOp::Install));
    assert_eq!(HookOp::parse("remove"), Some(HookOp::Remove));
    assert_eq!(HookOp::parse("nonsense"), None);
    assert_eq!(HookOp::Install.as_str(), "install");
}
