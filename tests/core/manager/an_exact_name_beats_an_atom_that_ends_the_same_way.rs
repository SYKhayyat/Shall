use super::*;

/// An exact name beats a qualified one even where both are on offer: the user wrote a name
/// this manager also has literally, and that is not a guess to be improved on.
#[tokio::test]
async fn an_exact_name_beats_an_atom_that_ends_the_same_way() {
    let both = catalogue(true, &["app-misc/jq", "jq"]);
    assert_eq!(
        both.lookup("jq").await.unwrap().map(|p| p.name),
        Some("jq".to_string())
    );
}
