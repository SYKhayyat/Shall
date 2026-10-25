use super::*;

#[test]
fn a_required_param_with_no_argument_is_a_loud_error() {
    // Never a silent empty string.
    let f = fixture(&[("workstation.txt", "param user\napt:vim\n")]);
    let err = expand_with_args(&f, "workstation", &[]).unwrap_err();
    assert!(err.what.contains("requires parameter `user`"), "{}", err);
    assert!(err.what.contains("workstation"), "{}", err);
}
