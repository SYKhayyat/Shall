use super::*;

#[test]
fn winget_search_parses_columns() {
    let header = wrow("Name", "Id", "Version", "Match", "Source");
    let sep = "-".repeat(110);
    let row = wrow(
        "Visual Studio Code",
        "Microsoft.VisualStudioCode",
        "1.85.0",
        "",
        "winget",
    );
    let input = format!("{}\n{}\n{}\n", header, sep, row);
    let res = parse_search("winget", &input);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "Microsoft.VisualStudioCode");
}
