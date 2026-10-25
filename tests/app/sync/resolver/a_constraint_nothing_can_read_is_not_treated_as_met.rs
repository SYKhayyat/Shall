use super::*;

/// **A pin nothing can parse must not read as satisfied.** The loose tier's `_ => false` is
/// what makes an unanswerable comparison a `Lacks` rather than a silent `Has` — the safe
/// direction, because reinstalling a package that was already right costs a command, and
/// skipping one that was wrong leaves the machine disagreeing with the config.
#[tokio::test]
async fn a_constraint_nothing_can_read_is_not_treated_as_met() {
    let r = repo(&[("modules/starter.txt", "")]);
    let resolver = StateResolver::new(&r.config, Arc::new(BackendRegistry::new()), false).await;

    assert!(
        !resolver.satisfies_constraint("1.2.3", "whatever-this-is"),
        "a constraint that is neither a range, the same string, nor comparable was \
         treated as satisfied"
    );
}
