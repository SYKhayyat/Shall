use super::*;

#[test]
fn parses_dotnet_tool_list() {
    let input = "Package Id      Version      Commands\n\
                 --------------------------------------\n\
                 dotnetsay       2.1.4        dotnetsay\n\
                 powershell      7.4.0        pwsh\n";
    let res = parse_dotnet_list(input).expect("this fixture parses");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "dotnetsay");
    assert_eq!(res[0].version.as_deref(), Some("2.1.4"));
    assert_eq!(res[1].name, "powershell");
    // header/separator rows must not leak through
    assert!(res
        .iter()
        .all(|p| p.name != "Package" && !p.name.starts_with('-')));
}
