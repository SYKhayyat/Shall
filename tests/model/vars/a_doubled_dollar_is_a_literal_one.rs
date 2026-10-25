use super::*;

#[test]
fn a_doubled_dollar_is_a_literal_one() {
    let v = resolve(&[top("price", "$$5", 1)]).unwrap();
    assert_eq!(v["price"], str_val("$5"));
}
