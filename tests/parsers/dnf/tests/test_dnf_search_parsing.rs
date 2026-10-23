use super::*;

#[test]
fn test_dnf_search_parsing() {
    let input =
        "htop.x86_64 : Interactive process viewer\npython3.noarch : Python programming language\n";
    let res = parse_dnf_search(input);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "htop");
    assert_eq!(
        res[0].properties.get("description").unwrap(),
        "Interactive process viewer"
    );
}
