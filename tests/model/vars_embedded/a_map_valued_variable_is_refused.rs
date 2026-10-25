use super::*;

#[test]
fn a_map_valued_variable_is_refused() {
    let err = run(r#"#{ nested: #{ a: 1 } }"#).unwrap_err();
    assert!(err.what.contains("cannot be a"), "{}", err);
}
