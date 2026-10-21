use super::*;

#[tokio::test]
async fn a_protected_name_stops_a_teardown_of_every_kind() {
    // V.26: protection is a refusal nothing overrides, and the ruling of 2026-07-28 is that
    // it covers resources as well as packages. One case per kind, because a guard that
    // holds for `link:` and not for `service:` is the shape this whole finding is about.
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(0); // count disabled, so only protection can object
    cfg.guard.protected_packages = vec!["keep".into()];

    for key in [
        "link:/home/u/keep",
        r"link:C:\Users\u\keep",
        "service:keep",
        "shim:keep",
        "schedule:keep",
        "setting:keep",
    ] {
        let report = inspect_removals(
            &cfg,
            &reg,
            &extras(&[key]),
            RemovalKind::Extra,
            &Reaping::new(),
            &mut EssentialsCache::default(),
        )
        .await;
        assert!(
            matches!(report.objections.as_slice(), [Objection::Protected { .. }]),
            "`{}` was not protected by `protected_packages = [\"keep\"]`: {:?}",
            key,
            report.objections
        );
    }

    // And the control: a name the rule does not match is still removable, or the assertion
    // above would pass for a guard that refuses everything.
    let report = inspect_removals(
        &cfg,
        &reg,
        &extras(&["link:/home/u/other"]),
        RemovalKind::Extra,
        &Reaping::new(),
        &mut EssentialsCache::default(),
    )
    .await;
    assert!(report.is_empty(), "{:?}", report.objections);
}
