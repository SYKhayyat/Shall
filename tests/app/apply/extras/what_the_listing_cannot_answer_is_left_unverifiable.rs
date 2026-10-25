use super::*;

/// What the listing cannot answer stays unanswered. A restart is a transition no listing
/// records; enablement is a second axis none of the shipped inits report. Answering either
/// from "is it running" would report converged on a machine that is not.
#[tokio::test]
async fn what_the_listing_cannot_answer_is_left_unverifiable() {
    let config = Arc::new(shall::config::Config::default());
    let reg = registry_listing_nginx();
    let exec = CommandExecutor::new(false, false);

    let (restarted, key) = service("nginx", "status", "restarted");
    assert_eq!(
        in_effect(&config, &reg, &exec, &restarted, &key).await,
        None
    );

    let (enabled, key) = service("nginx", "enabled", "true");
    assert_eq!(in_effect(&config, &reg, &exec, &enabled, &key).await, None);

    // Enablement declared *alongside* a status is still unanswered: the status half being
    // satisfied says nothing about the half that is not.
    let mut opts = Options::default();
    opts.insert("status".to_string(), "running");
    opts.insert("enabled".to_string(), "true");
    let both = Statement::Service("nginx".to_string(), opts);
    let key = ExtraKey::new(shall::config::grammar::ResourceKind::Service, "nginx");
    assert_eq!(
        in_effect(&config, &reg, &exec, &both, &key).await,
        None,
        "running is satisfied and enabled is unknown — the line as a whole is unknown"
    );
}
