use super::*;

#[test]
fn a_name_with_no_version_survives_whole() {
    let res = parse_dash_version_list("tree\n", "apk").expect("this fixture parses");
    assert_eq!(res[0].name, "tree");
    assert_eq!(res[0].version, None);
}
