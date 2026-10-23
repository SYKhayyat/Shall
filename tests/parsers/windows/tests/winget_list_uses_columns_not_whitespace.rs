use super::*;

#[test]
fn winget_list_uses_columns_not_whitespace() {
    let res = parse_installed("winget", &winget_list_fixture()).expect("this fixture parses");
    assert_eq!(
        res.len(),
        8,
        "eight data rows, no header and no dashed rule"
    );

    // multi-word display name must NOT corrupt identity/version
    let sevenz = res
        .iter()
        .find(|p| p.name == "7zip.7zip")
        .expect("7zip.7zip present");
    assert_eq!(sevenz.version.as_deref(), Some("25.01"));

    // An ARP id: backslashes and braces, parsed whole. 185 of the 278 names this machine
    // reports are of that shape, and a truncated one names a package that does not exist.
    let affinity = res
        .iter()
        .find(|p| p.name.starts_with("ARP\\Machine\\X64\\{8BD2A40D"))
        .expect("the braced ARP id survived");
    assert_eq!(affinity.version.as_deref(), Some("2.6.5.3782"));

    // none of the old garbage fragments should appear as packages
    for bad in ["Studio", "(x64)", "25.01", "Name", "HDR"] {
        assert!(
            !res.iter().any(|p| p.name == bad),
            "unexpected garbage row: {bad}"
        );
    }
}
