use super::*;

#[test]
fn test_bun_list_parsing() {
    // Real `bun pm ls -g` shape: header path line + tree rows, incl. a scoped pkg.
    let input = "/root/.bun/install/global node_modules (2)\n\
                 ├── cowsay@1.6.0\n\
                 └── @scope/tool@2.3.4\n";
    let res = parse_bun_list(input);
    assert_eq!(
        res.len(),
        2,
        "header line must be skipped, both pkgs parsed"
    );
    assert_eq!(res[0].name, "cowsay");
    assert_eq!(res[0].version, Some("1.6.0".into()));
    // Scoped names keep their leading '@'; only the trailing @version splits off.
    assert_eq!(res[1].name, "@scope/tool");
    assert_eq!(res[1].version, Some("2.3.4".into()));
}

#[test]
fn test_bun_list_parsing_with_installed_suffix_header() {
    // Newer bun appends the word to its count (#129): the header is `(N installed)`. A
    // populated listing under that header must still parse, and the count must stay a
    // zero-detection detail — nonzero never reads as empty.
    let input = "/home/shall/.bun/install/global node_modules (2 installed)\n\
                 ├── cowsay@1.6.0\n\
                 └── @scope/tool@2.3.4\n";
    let res = parse_installed("bun", input).expect("the new header shape parses");
    assert_eq!(res.len(), 2, "header skipped, both pkgs parsed");
    assert_eq!(res[0].name, "cowsay");
    assert_eq!(res[1].name, "@scope/tool");

    // A nonzero count in either shape is not an empty machine, whatever the rows say.
    let unreadable = "/home/shall/.bun/install/global node_modules (2 installed)\n";
    assert!(
        parse_installed("bun", unreadable).is_err(),
        "a count of 2 with no parseable rows is a format change, not emptiness"
    );
}
