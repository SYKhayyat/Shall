use super::*;

/// The exit-code axis added for `Q41` must not have made every manager's codes benign.
#[test]
fn a_transient_code_is_not_thereby_a_benign_one() {
    let p = winget();
    let internal = 0x8A15_0001_u32 as i32;
    assert_eq!(
        p.retryability_of(Some(internal), ""),
        Retryability::Transient
    );
    assert!(
        !p.is_benign(Some(internal)),
        "worth retrying and `not a failure` are different claims"
    );
}
