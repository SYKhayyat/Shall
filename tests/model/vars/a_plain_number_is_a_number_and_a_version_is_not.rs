use super::*;

#[test]
fn a_plain_number_is_a_number_and_a_version_is_not() {
    let v = resolve(&[
        top("count", "3", 1),
        top("ver", "1.6.0", 2),
        top("ratio", "1.5", 3),
    ])
    .unwrap();
    assert_eq!(v["count"], Value::Num(3.0));
    assert_eq!(v["ver"], str_val("1.6.0"));
    assert_eq!(v["ratio"], Value::Num(1.5));
}
