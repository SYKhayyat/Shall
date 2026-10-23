use super::*;

#[test]
fn test_pacman_search_parsing() {
    let input = "core/bash 5.1.016-1 (base)\n    The GNU Bourne Again Shell\nextra/vim 9.0.1583-1\n    Vi Improved, a highly configurable, improved version of the Vi real-time editor\n";
    let res = parse_search(input);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "bash");
    assert_eq!(
        res[0].properties.get("description").unwrap(),
        "The GNU Bourne Again Shell"
    );
    assert_eq!(res[1].name, "vim");
}
