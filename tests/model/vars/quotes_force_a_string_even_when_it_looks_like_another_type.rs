use super::*;

#[test]
fn quotes_force_a_string_even_when_it_looks_like_another_type() {
    let v = resolve(&[top("flag", "\"true\"", 1), top("n", "\"5\"", 2)]).unwrap();
    assert_eq!(v["flag"], str_val("true"));
    assert_eq!(v["n"], str_val("5"));
}
