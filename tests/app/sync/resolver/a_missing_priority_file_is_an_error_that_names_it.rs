use super::*;

#[tokio::test]
async fn a_missing_priority_file_is_an_error_that_names_it() {
    // Not a detected default. Which package managers this machine uses is a thing you
    // declare, and guessing it is what V.15 exists to stop.
    let r = repo(&[("active", "Work\n"), ("profiles/Work", "apt:curl\n")]);
    let err = resolve(&r).await.unwrap_err().to_string();
    assert!(err.contains("priority"), "{}", err);
    assert!(err.contains("one per line"), "{}", err);
}
