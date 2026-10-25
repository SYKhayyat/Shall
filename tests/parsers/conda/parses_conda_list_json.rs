use super::*;

#[test]
fn parses_conda_list_json() {
    let input = r#"[
        {"name": "numpy", "version": "1.26.0", "channel": "defaults"},
        {"name": "pandas", "version": "2.1.1", "channel": "defaults"}
    ]"#;
    let res = parse_conda_list(input).expect("this fixture parses");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "numpy");
    assert_eq!(res[0].version.as_deref(), Some("1.26.0"));
    assert_eq!(res[1].name, "pandas");
}
