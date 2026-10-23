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
