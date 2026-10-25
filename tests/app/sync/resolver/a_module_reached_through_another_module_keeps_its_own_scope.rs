use super::*;

#[tokio::test]
async fn a_module_reached_through_another_module_keeps_its_own_scope() {
    let r = repo(&[
        ("priority", "apt\n"),
        ("active", "Work\n"),
        ("profiles/Work", "use dev\n"),
        ("modules/dev.txt", "use base\napt:curl\n"),
        ("modules/base.txt", "apt:jq\n"),
    ]);
    let map = resolve(&r).await.unwrap();
    let jq = map
        .get("apt")
        .unwrap()
        .iter()
        .find(|s| s.name == "jq")
        .unwrap();
    let scopes = jq.options.all("__scopes");
    assert!(scopes.iter().any(|s| s == "module:base"), "{scopes:?}");
    assert!(scopes.iter().any(|s| s == "profile:Work"), "{scopes:?}");
}
