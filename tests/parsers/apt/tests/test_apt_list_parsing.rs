use super::*;

#[test]
fn test_apt_list_parsing() {
    let input = "installed bash 5.1-6ubuntu1\ninstalled coreutils 8.32-4.1ubuntu1\n";
    let res = parse_list(input).expect("this fixture parses");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "bash");
    assert_eq!(res[0].version, Some("5.1-6ubuntu1".into()));
}
