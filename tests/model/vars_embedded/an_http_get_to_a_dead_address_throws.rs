use super::*;

#[test]
fn an_http_get_to_a_dead_address_throws() {
    // Port 1 refuses fast; no live network needed. Proves failure is loud, not empty.
    let err = run(r#"#{ x: http_get("http://127.0.0.1:1/") }"#).unwrap_err();
    assert!(
        err.what.contains("did not run") || err.what.contains("http"),
        "{}",
        err
    );
}
