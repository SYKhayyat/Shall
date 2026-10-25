use super::*;

#[test]
fn a_listing_from_a_client_alone_keeps_its_rows() {
    let kept: Vec<String> = one_row_per_shared_database(
        vec![pkg("yay", "jq"), pkg("yay", "bash")],
        &ForeignSets::default(),
    )
    .into_iter()
    .map(|p| p.name)
    .collect();
    assert_eq!(kept, vec!["jq", "bash"]);
}
