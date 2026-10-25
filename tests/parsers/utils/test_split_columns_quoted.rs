use super::*;

#[test]
fn test_split_columns_quoted() {
    let input = "Microsoft.PowerShell \"7.3.4 (x64)\" installed";
    let cols = split_columns(input);
    assert_eq!(cols.len(), 3);
    assert_eq!(cols[1], "7.3.4 (x64)");
}
