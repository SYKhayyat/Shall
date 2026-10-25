use super::*;

#[tokio::test]
async fn a_package_is_scoped_to_its_module_and_to_the_profile_that_reaches_it() {
    // What `upgrade --module dev` and `upgrade --profile Work` match on.
    let r = repo(&[
        ("priority", "apt\n"),
        ("active", "Work\n"),
        ("profiles/Work", "use dev\n"),
        ("modules/dev.txt", "apt:curl\n"),
    ]);
    let map = resolve(&r).await.unwrap();
    let curl = &map.get("apt").unwrap()[0];
    let scopes = curl.options.all("__scopes");
    assert!(scopes.iter().any(|s| s == "module:dev"), "{scopes:?}");
    assert!(scopes.iter().any(|s| s == "profile:Work"), "{scopes:?}");
    // And `__source` stays the human answer to "where is this line?".
    assert!(curl.options.one("__source").unwrap().contains("dev.txt:1"));
}
