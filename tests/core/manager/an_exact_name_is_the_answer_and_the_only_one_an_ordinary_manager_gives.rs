use super::*;

/// The exact name wins wherever it appears, and on a manager that qualifies nothing it is
/// the only rule there is.
#[tokio::test]
async fn an_exact_name_is_the_answer_and_the_only_one_an_ordinary_manager_gives() {
    let plain = catalogue(false, &["jq-mode", "jq", "jqp"]);
    assert_eq!(
        plain.lookup("jq").await.unwrap().map(|p| p.name),
        Some("jq".to_string())
    );

    // The same catalogue without the exact name: an ordinary manager says no rather than
    // guessing at a near miss, which is what stops a typo becoming an install.
    let near = catalogue(false, &["jq-mode", "jqp"]);
    assert_eq!(near.lookup("jq").await.unwrap(), None);

    // And a qualified name is not a near miss to be found by a manager that does not
    // qualify: `foo/bar` and `bar` are two packages there.
    let unqualifying = catalogue(false, &["app-misc/jq"]);
    assert_eq!(unqualifying.lookup("jq").await.unwrap(), None);
}
