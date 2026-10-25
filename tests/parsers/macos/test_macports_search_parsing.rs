use super::*;

#[test]
fn test_macports_search_parsing() {
    let input = "git @2.39.0 (devel, net): Distributed version control system\n\
                 Found 1 ports.\n";
    let res = parse_macports_search(input);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "git");
    assert_eq!(res[0].version.as_deref(), Some("2.39.0"));
    assert_eq!(
        res[0].properties.get("description").map(String::as_str),
        Some("Distributed version control system")
    );
}
