use super::*;

#[test]
fn an_argument_that_names_no_parameter_is_an_error() {
    let f = fixture(&[("workstation.txt", "param user\napt:vim\n")]);
    let err = expand_with_args(&f, "workstation", &[("gpu", "nvidia")]).unwrap_err();
    assert!(err.what.contains("has no parameter `gpu`"), "{}", err);
}
