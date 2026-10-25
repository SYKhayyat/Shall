use super::*;

#[test]
fn test_apt_search_parsing() {
    let input = "htop - interactive processes viewer\nvim - Vi IMproved - enhanced vi editor\n";
    let res = parse_search(input);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "htop");
    assert_eq!(
        res[0].properties.get("description").unwrap(),
        "interactive processes viewer"
    );
}
