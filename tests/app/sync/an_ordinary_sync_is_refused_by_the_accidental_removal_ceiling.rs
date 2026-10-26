use super::*;

#[tokio::test]
async fn an_ordinary_sync_is_refused_by_the_accidental_removal_ceiling() {
    let mut config = Config::sandboxed(&config_dir("ordinary-removal-ceiling"));
    config.guard.max_removals = 1;
    let (engine, mock) = engine(config).await;
    let changes = removal_changes(&[("brew", "old-brew"), ("cargo", "old-cargo")]);

    let error = engine
        .sync(changes, GuardScope::Sync)
        .await
        .expect_err("the ordinary sync removed past its ceiling");

    assert!(
        error.to_string().contains("max_removals"),
        "the refusal does not name the ceiling: {error}"
    );
    assert!(
        mock.get_calls().await.is_empty(),
        "the refused sync removed a package"
    );
}
