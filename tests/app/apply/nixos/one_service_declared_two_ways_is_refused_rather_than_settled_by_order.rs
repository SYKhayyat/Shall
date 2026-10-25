use super::*;

/// Two modules declaring one service two ways is the same contradiction one line apart. It
/// must not be settled by load order — on systemd the last write wins and the machine ends
/// up in whichever state was applied last, which is exactly the drift this tool removes.
#[test]
fn one_service_declared_two_ways_is_refused_rather_than_settled_by_order() {
    let err = declared(&state(vec![
        Statement::Service("nginx".into(), opts(&[("enabled", "true")])),
        Statement::Service("nginx".into(), opts(&[("enabled", "false")])),
    ]))
    .expect_err("two answers for one attribute")
    .to_string();
    assert!(err.contains("nginx"), "{err}");
    // The same name declared the same way twice is not a contradiction and must pass.
    declared(&state(vec![
        Statement::Service("nginx".into(), opts(&[("enabled", "true")])),
        Statement::Service("nginx".into(), Options::default()),
    ]))
    .expect("two lines agreeing is not a conflict");
}
