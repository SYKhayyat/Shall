use super::*;

#[test]
fn test_mas_list_parsing() {
    let input = "497799835 Xcode (14.3.1)\n1284863847 Unarchiver (3.35.2)\n";
    let res = parse_mas_list(input).expect("this fixture parses");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "497799835");
    assert_eq!(res[0].version, Some("14.3.1".into()));
    assert_eq!(res[0].properties.get("human_name").unwrap(), "Xcode");
}
