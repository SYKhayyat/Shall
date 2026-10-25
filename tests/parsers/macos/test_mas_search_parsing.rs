use super::*;

#[test]
fn test_mas_search_parsing() {
    let input = "497799835 Xcode\n1284863847 The Unarchiver\n";
    let res = parse_mas_search(input);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "497799835");
    assert_eq!(
        res[1].properties.get("human_name").unwrap(),
        "The Unarchiver"
    );
}
