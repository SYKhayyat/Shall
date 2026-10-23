use super::*;

#[test]
fn test_choco_list_parsing() {
    let input = "git|2.40.1\ncurl|8.1.2\n";
    let res = parse_installed("choco", input).expect("this fixture parses");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "git");
    assert_eq!(res[1].version, Some("8.1.2".into()));
}
