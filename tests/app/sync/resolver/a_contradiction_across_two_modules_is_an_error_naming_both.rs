use super::*;

#[tokio::test]
async fn a_contradiction_across_two_modules_is_an_error_naming_both() {
    // Part IV requires this proof, through the seam and not just in the model.
    let r = repo(&[
        ("priority", "apt\n"),
        ("active", "Work\n"),
        ("profiles/Work", "use a\nuse b\n"),
        ("modules/a.txt", "apt:jq@version=1.6\n"),
        ("modules/b.txt", "apt:jq@version=1.7\n"),
    ]);
    let err = resolve(&r).await.unwrap_err().to_string();
    assert!(err.contains("a.txt"), "{}", err);
    assert!(err.contains("b.txt"), "{}", err);
}
