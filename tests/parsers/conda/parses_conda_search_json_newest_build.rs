use super::*;

#[test]
fn parses_conda_search_json_newest_build() {
    let input = r#"{
        "numpy": [
            {"name": "numpy", "version": "1.25.0"},
            {"name": "numpy", "version": "1.26.0"}
        ]
    }"#;
    let res = parse_conda_search(input);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "numpy");
    assert_eq!(res[0].version.as_deref(), Some("1.26.0"));
}
