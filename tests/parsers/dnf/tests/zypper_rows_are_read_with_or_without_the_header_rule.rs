use super::*;

#[test]
fn zypper_rows_are_read_with_or_without_the_header_rule() {
    // This parser is zypper's installed lister as well as its search, so returning
    // nothing is not a bad search result — it is "nothing is installed", which is a
    // mass-removal input. Waiting for a `---` rule before reading anything meant one
    // missing rule produced exactly that.
    let with_rule = "S | Name | Summary | Type\n\
                     --+------+---------+-----\n\
                     i | jq   | JSON    | package\n\
                       | htop | Viewer  | package\n";
    let without_rule = "S | Name | Summary | Type\n\
                        i | jq   | JSON    | package\n\
                          | htop | Viewer  | package\n";
    for (label, input) in [("with", with_rule), ("without", without_rule)] {
        let res = parse_zypper_search(input).expect("this fixture parses");
        assert_eq!(res.len(), 2, "{} the rule", label);
        assert_eq!(res[0].name, "jq");
        assert_eq!(
            res[0].properties.get("installed").map(String::as_str),
            Some("true")
        );
        assert_eq!(res[1].name, "htop");
        assert_eq!(res[1].properties.get("installed"), None);
    }
}
