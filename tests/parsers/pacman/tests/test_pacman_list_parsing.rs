use super::*;

#[test]
fn test_pacman_list_parsing() {
    let input = "bash 5.1.016-1\nlinux 6.3.5.arch1-1\n";
    let res = parse_list(input).expect("this fixture parses");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "bash");
    assert_eq!(res[1].version, Some("6.3.5.arch1-1".into()));
}
