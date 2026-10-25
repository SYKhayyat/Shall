use super::*;

/// A body that is not JSON is an error the script can see, not a silent empty map.
#[test]
fn a_body_that_is_not_json_is_an_error() {
    let engine = engine("test");
    let err = engine
        .eval::<rhai::Dynamic>(r#"parse_json("not json at all")"#)
        .expect_err("nonsense parsed as JSON");
    assert!(
        err.to_string().contains("parse_json"),
        "the failure does not say which builtin produced it: {err}"
    );
}
