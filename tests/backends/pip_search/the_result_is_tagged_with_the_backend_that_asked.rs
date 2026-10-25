use super::*;

/// The backend tag comes from the row asking, not from the word `pip` — the same reason
/// `node_registry` takes one and three Node managers share it.
#[test]
fn the_result_is_tagged_with_the_backend_that_asked() {
    let json: serde_json::Value =
        serde_json::from_str(r#"{"info": {"name": "ruff", "version": "0.16.2"}}"#).unwrap();
    assert_eq!(parse_pypi(&json, "ruff", "uv").backend, "uv");
}
