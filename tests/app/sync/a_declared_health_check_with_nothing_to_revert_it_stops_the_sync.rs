use super::*;

/// **A health check that cannot be reverted stops the sync before it starts.**
///
/// Found by the mutation gate: `require_revert_path` replaced with `Ok(())` survived the whole
/// suite. **It is the check that runs before anything is installed**, and replacing it with
/// `Ok(())` is "the check always passes" — so a machine with health checks declared and no
/// snapshot provider would be *changed* anyway, and the breakage the checks exist to catch
/// would be found out afterwards, when it is too late to undo.
///
/// A health check that cannot revert reports the breakage and leaves it in place — strictly
/// worse than not checking, because you are told the machine is broken and given no way back.
/// **The only moment that fact is actionable is before the change.**
#[tokio::test]
async fn a_declared_health_check_with_nothing_to_revert_it_stops_the_sync() {
    let (engine, _mock) = engine(config_with_health("unrevertable", &["port:8080"])).await;

    let refused = engine.require_revert_path(&no_changes());
    assert!(
        refused.is_err(),
        "a declared health check on a machine that cannot revert was allowed to start: {:?}",
        refused
    );
    let message = refused.unwrap_err().to_string();
    assert!(
        message.contains("health check") && message.contains("snapshot provider"),
        "the refusal must say what is wrong and what would fix it: {message}"
    );
}
