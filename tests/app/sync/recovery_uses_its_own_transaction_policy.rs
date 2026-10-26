use super::*;
use shall::core::ContinuePast;

#[tokio::test]
async fn recovery_uses_its_own_transaction_policy() {
    let mut config = Config::sandboxed(&config_dir("heal-policy"));
    config.max_parallel = 3;
    let (engine, _mock) = engine(config).await;

    let policy = engine.heal_transaction_config();

    assert_eq!(policy.max_concurrent, 3);
    assert!(!policy.auto_rollback);
    assert_eq!(policy.continue_past, ContinuePast::AnyFailure);
}
