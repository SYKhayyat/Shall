use super::*;

#[test]
fn scoop_list_ingests_no_header_or_separator() {
    let names: Vec<String> = parse_scoop_list(&scoop_list_fixture())
        .expect("the fixture has a header row")
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert!(
        !names
            .iter()
            .any(|n| n == "Name" || n.starts_with('-') || n == "Installed"),
        "{:?}",
        names
    );
}
