use super::*;

#[test]
fn test_cargo_list_parsing() {
    let input = "ripgrep v13.0.0:\nexa v0.10.1:\n";
    let res = parse_cargo_list(input);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "ripgrep");
    assert_eq!(res[1].version, Some("0.10.1".into()));
}
