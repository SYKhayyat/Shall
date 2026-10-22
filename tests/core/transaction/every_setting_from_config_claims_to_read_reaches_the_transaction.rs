use super::*;

/// Every setting this constructor exists to carry is actually carried.
///
/// The doc comment on `from_config` records why it exists: `max_concurrent` had been left
/// at the `patient()` default and nobody noticed, because a field that silently falls back
/// to a sensible number looks exactly like a field that was read. `..Self::patient()` makes
/// that the failure mode for every line in the struct â€” deleting `max_concurrent`, `purge`
/// or `manager_lock_wait` left the suite green, which is the same defect the constructor
/// was written to prevent, three more times.
///
/// `max_concurrent` is asserted at two different values rather than one. The fallback is
/// this machine's parallelism, so a single value can agree with it by luck on whatever host
/// happens to run the test; two values cannot both be it.
#[test]
fn every_setting_from_config_claims_to_read_reaches_the_transaction() {
    let base = shall::config::Config::default;

    for parallel in [1usize, 64] {
        let mut config = base();
        config.max_parallel = parallel;
        assert_eq!(
            TransactionConfig::from_config(&config).max_concurrent,
            parallel,
            "max_parallel = {parallel} did not reach max_concurrent"
        );
    }
    let mut none = base();
    none.max_parallel = 0;
    assert_eq!(
        TransactionConfig::from_config(&none).max_concurrent,
        1,
        "nought parallelism is one worker, not none"
    );

    // The per-package budget and its floor. **This key was declared, documented and
    // defaulted and read nowhere** — the exact shape
    // `a_setting_the_config_declares_is_a_setting_that_acts_tests` is named for, and that gate
    // caught it on CI before I looked. A gate doing its job is the reason to keep one.
    for secs in [1u64, 900] {
        let mut config = base();
        config.manager_command_timeout_secs = secs;
        assert_eq!(
            TransactionConfig::from_config(&config).node_timeout,
            std::time::Duration::from_secs(secs),
            "`manager_command_timeout_secs = {secs}` did not reach the per-package budget"
        );
    }
    assert_eq!(
        TransactionConfig::from_config(&base()).node_timeout,
        std::time::Duration::from_secs(300),
        "the default budget is not the default the key documents"
    );
    let mut floored = base();
    floored.manager_command_timeout_secs = 0;
    assert_eq!(
        TransactionConfig::from_config(&floored).node_timeout,
        std::time::Duration::from_secs(1),
        "a zero budget kills every manager command on arrival"
    );

    // Purge is an OR of a persistent setting and a this-run flag, and each half must be
    // able to turn it on alone â€” `&&` reads identically until you ask one of them by itself.
    let mut persistent = base();
    persistent.remove.purge = true;
    assert!(
        TransactionConfig::from_config(&persistent).purge,
        "`[remove] purge = true` alone must purge"
    );
    let mut this_run = base();
    this_run.purge_this_run = true;
    assert!(
        TransactionConfig::from_config(&this_run).purge,
        "`--purge` alone must purge"
    );
    assert!(
        !TransactionConfig::from_config(&base()).purge,
        "neither set is not a purge"
    );

    let mut keep_going = base();
    keep_going.keep_going_this_run = true;
    assert_eq!(
        TransactionConfig::from_config(&keep_going).continue_past,
        ContinuePast::AnyFailure,
        "`--keep-going` has to reach the transaction, or the flag is decoration"
    );

    // All three states of `M2`'s wiring. The flag OUTRANKS the key rather than combining
    // with it: somebody at a keyboard said `--keep-going`, and the key is only what the
    // machine does when nobody said anything.
    assert_eq!(
        TransactionConfig::from_config(&base()).continue_past,
        ContinuePast::ClassifiedPassing,
        "`[sync] continue_past_transient` defaults on, so a stock machine finishes what \
         it can past a failure Shall classed as passing"
    );
    let mut all_or_nothing = base();
    all_or_nothing.sync.continue_past_transient = false;
    assert_eq!(
        TransactionConfig::from_config(&all_or_nothing).continue_past,
        ContinuePast::Nothing,
        "turning the key off has to reach the transaction, or the key is decoration"
    );
    let mut both = all_or_nothing.clone();
    both.keep_going_this_run = true;
    assert_eq!(
        TransactionConfig::from_config(&both).continue_past,
        ContinuePast::AnyFailure,
        "the flag outranks the key: a run told to keep going keeps going"
    );

    // **`M3`'s key, which a nightly mutant found unasserted.** Deleting
    // `batch_recovery: config.sync.batch_recovery` from this struct expression leaves the
    // field at `patient()`'s `Off`, and every test still passed - so the key could have
    // silently done nothing on every machine and nothing would have said.
    assert_eq!(
        TransactionConfig::from_config(&base()).batch_recovery,
        BatchRecovery::Bisect,
        "`[sync] batch_recovery` defaults to bisecting, so a failed batch is narrowed"
    );
    let mut no_narrowing = base();
    no_narrowing.sync.batch_recovery = BatchRecovery::Off;
    assert_eq!(
        TransactionConfig::from_config(&no_narrowing).batch_recovery,
        BatchRecovery::Off,
        "turning the key off has to reach the transaction, or the key is decoration"
    );
    let mut every = base();
    every.sync.batch_recovery = BatchRecovery::Every;
    assert_eq!(
        TransactionConfig::from_config(&every).batch_recovery,
        BatchRecovery::Every,
        "and the third setting reaches it too, or one of three values is unreachable"
    );

    // A wait no default would produce, so the fallback cannot pass for the setting.
    let mut waiting = base();
    waiting.manager_lock_wait_secs = 4_321;
    assert_eq!(
        TransactionConfig::from_config(&waiting).manager_lock_wait,
        Duration::from_secs(4_321),
        "manager_lock_wait_secs did not reach the transaction"
    );
}
