use super::*;

/// On a manager that qualifies, one matching atom resolves — and the *atom* comes back,
/// because that is the string the manager will want and will report.
#[tokio::test]
async fn one_matching_atom_resolves_and_answers_with_the_atom() {
    let portage = catalogue(true, &["app-emacs/jq-mode", "app-misc/jq"]);
    assert_eq!(
        portage.lookup("jq").await.unwrap().map(|p| p.name),
        Some("app-misc/jq".to_string()),
        "`jq-mode` is not a package called jq, and `app-misc/jq` is"
    );
}
