use super::*;

#[test]
fn an_unknown_failure_is_left_alone() {
    let e = Error::CommandFailed {
        message: "`mix` failed: something".into(),
        retry: Retryability::Unknown,
        absent_name: false,
    };
    let out = falsify_transience(e, 3);
    assert_eq!(out.retryability(), Retryability::Unknown);
    assert!(!out.to_string().contains("did not change"));
}
