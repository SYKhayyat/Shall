use super::*;

#[test]
fn winget_noteworthy_codes_are_benign() {
    let winget = exit_policy::winget();
    for code in [-1978335189, -1978335212, -1978335215] {
        assert!(winget.is_benign(Some(code)), "winget {code}");
    }
    assert!(!winget.is_benign(Some(1)));
    assert!(!winget.is_benign(None));
    assert!(executor_for(exit_policy::winget())
        .ensure_status("winget", finished(1, "", ""))
        .is_err());
}
