use super::*;

#[test]
fn apk_search_output_yields_the_bare_name() {
    // `apk search -v` adds ` - description` after the token. Splitting on whitespace
    // alone left the name as `jq-1.7.1-r0`, which never equals the name a line asked
    // for — so apk answered "I don't have it" for everything it had.
    let res = parse_dash_version_list("jq-1.7.1-r0 - Command-line JSON processor\n", "apk")
        .expect("this fixture parses");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "jq");
    assert_eq!(res[0].version.as_deref(), Some("1.7.1-r0"));
}
