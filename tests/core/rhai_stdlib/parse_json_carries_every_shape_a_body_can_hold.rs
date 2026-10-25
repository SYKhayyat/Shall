use super::*;

/// **`parse_json` is the only way a script sees a network answer, and its converter had no
/// test.** `json_to_dynamic` is a six-arm recursion — null, bool, number, string, array,
/// object — and every arm decides what a resolved variable becomes, which decides which
/// packages get declared. Driven through a real `Engine` rather than by calling the
/// converter directly, because registration is half of what could be wrong.
#[test]
fn parse_json_carries_every_shape_a_body_can_hold() {
    let engine = engine("test");
    let out: rhai::Map = engine
        .eval(r#"parse_json("{\"name\":\"rg\",\"vers\":[1,2],\"meta\":{\"ok\":true},\"none\":null,\"f\":1.5}")"#)
        .expect("the body did not parse");

    assert_eq!(out["name"].clone().into_string().unwrap(), "rg");
    assert!(out["meta"].is_map(), "a nested object did not become a map");
    assert!(out["none"].is_unit(), "JSON null did not become ()");

    let vers = out["vers"].clone().into_array().expect("array");
    assert_eq!(vers.len(), 2);
    assert_eq!(
        vers[0].as_int().unwrap(),
        1,
        "an integer inside an array became something else"
    );
    assert!(
        (out["f"].as_float().unwrap() - 1.5).abs() < f64::EPSILON,
        "a fractional number did not survive as a float"
    );
}
