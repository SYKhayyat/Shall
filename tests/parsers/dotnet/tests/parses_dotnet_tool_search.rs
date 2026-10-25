use super::*;

#[test]
fn parses_dotnet_tool_search() {
    let input = "Package ID        Latest Version      Authors      Downloads\n\
                 ----------------  ------------------  -----------  ---------\n\
                 dotnet-ef         8.0.0               Microsoft    123456\n";
    let res = parse_dotnet_search(input).expect("this fixture parses");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "dotnet-ef");
    assert_eq!(res[0].version.as_deref(), Some("8.0.0"));
}
