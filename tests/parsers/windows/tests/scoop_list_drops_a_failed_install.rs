use super::*;

#[test]
fn scoop_list_drops_a_failed_install() {
    let res = parse_scoop_list(&scoop_list_fixture()).expect("the fixture has a header row");
    let names: Vec<&str> = res.iter().map(|p| p.name.as_str()).collect();
    assert!(
        !names.contains(&"jq"),
        "a row whose Info says the install failed is not an installed package: {:?}",
        names
    );
    assert!(
        names.contains(&"7zip") && names.contains(&"ripgrep"),
        "{:?}",
        names
    );
}
