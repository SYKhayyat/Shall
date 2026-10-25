use super::*;

#[test]
fn a_bracketed_value_is_a_list_of_typed_elements() {
    let v = resolve(&[
        top("tags", "[travel, work]", 1),
        top("ports", "[22, 80]", 2),
    ])
    .unwrap();
    assert_eq!(
        v["tags"],
        Value::List(vec![str_val("travel"), str_val("work")])
    );
    assert_eq!(
        v["ports"],
        Value::List(vec![Value::Num(22.0), Value::Num(80.0)])
    );
}
