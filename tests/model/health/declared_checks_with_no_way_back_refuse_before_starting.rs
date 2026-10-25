use super::*;

/// 7f's exit condition: with no snapshot provider, a declared health check stops the run
/// **before it starts**.
#[test]
fn declared_checks_with_no_way_back_refuse_before_starting() {
    let msg = refusal_if_unrevertable(&one_check(), false, false)
        .expect("this must refuse — there is no way back");
    assert!(msg.contains("refusing to start"), "{}", msg);
}
