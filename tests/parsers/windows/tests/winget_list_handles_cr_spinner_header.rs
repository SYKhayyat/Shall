use super::*;

#[test]
fn winget_list_handles_cr_spinner_header() {
    // Prepend the bare-\r progress spinner winget draws before the header.
    let fixture = winget_list_fixture();
    let with_spinner = format!("  - \r  \\ \r  / \r{}", fixture);
    let res = parse_installed("winget", &with_spinner).expect("this fixture parses");
    assert_eq!(res.len(), 8);
    assert!(res.iter().any(|p| p.name == "7zip.7zip"));
}
