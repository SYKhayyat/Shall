use super::*;

#[test]
fn a_shell_positional_is_not_a_variable_reference() {
    let v = resolve(&[top("cmd", "awk '{print $1}'", 1)]).unwrap();
    assert_eq!(v["cmd"], str_val("awk '{print $1}'"));
}
