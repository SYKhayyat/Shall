use super::*;

#[test]
fn two_providers_and_no_choice_is_a_loud_error() {
    let tmp = tmp();
    let dir = tmp.path();
    std::fs::write(dir.join("vars"), "role = desktop\n").unwrap();
    std::fs::write(dir.join("vars.py"), "print()\n").unwrap();
    let err = select(dir, &None).unwrap_err();
    assert!(err.what.contains("more than one"), "{}", err);
    assert!(err.what.contains("vars.py"), "{}", err);
}
