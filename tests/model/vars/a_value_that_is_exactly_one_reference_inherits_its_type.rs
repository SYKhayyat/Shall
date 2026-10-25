use super::*;

#[test]
fn a_value_that_is_exactly_one_reference_inherits_its_type() {
    let v = resolve(&[
        top("tags", "[a, b]", 1),
        top("alias", "$tags", 2),
        top("count", "3", 3),
        top("n", "${count}", 4),
    ])
    .unwrap();
    assert_eq!(v["alias"], Value::List(vec![str_val("a"), str_val("b")]));
    assert_eq!(
        v["n"],
        Value::Num(3.0),
        "a sole reference keeps the number type"
    );
}
