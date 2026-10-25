use super::*;

#[tokio::test]
async fn an_unreached_broken_module_is_never_parsed() {
    // II.3: Shall only parses what the active profiles reach. `shall check` is the
    // command that parses everything.
    let r = repo(&[
        ("priority", "apt\n"),
        ("active", "Work\n"),
        ("profiles/Work", "use base\n"),
        ("modules/base.txt", "apt:curl\n"),
        ("modules/broken.txt", "!!! not a statement !!!\n"),
    ]);
    assert!(resolve(&r).await.is_ok());
}
