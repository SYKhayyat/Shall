//! **`MockExecutor` had two ledgers and one reader, and the unread one was the dangerous half.**
//!
//! `unmatched_registrations` — a stub the test wrote down and the product never ran — has no
//! innocent reading and has failed at `Drop` since the since-deleted `e2e_tests.rs`, which
//! registered `brew install {name}` against a product emitting `brew install -- neovim`. Its
//! twin, `unstubbed`, records every command that ran with *nothing* registered, and was read by
//! nothing at all: the test double kept a complete record of every question the suite failed to
//! prepare an answer for, and no test, gate or build had ever asked for it.
//!
//! These live in `tests/` rather than beside the double because `src/core/executor.rs` carries a
//! recorded line ceiling and this is 150 lines of assertions about it rather than part of it. The
//! **reasoning** is `V.215`; the parts that stay in the source are the two doc comments that
//! state a constraint a caller cannot see from the signature.
use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use shall::core::executor::{DryRunOutput, ExecutionLayer, MockExecutor};

fn mock() -> Arc<MockExecutor> {
    Arc::new(MockExecutor::new(Arc::new(DashMap::new())))
}

fn args(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

async fn run(m: &MockExecutor, cmd: &str, argv: &[&str]) {
    m.execute(cmd, &args(argv), &HashMap::new())
        .await
        .expect("the empty-success default is a success, which is the whole problem");
}

#[tokio::test]
async fn the_ledger_records_a_fall_through_and_the_accessor_reads_it() {
    let m = mock();
    run(&m, "brew", &["list", "--versions"]).await;
    assert_eq!(
        m.unstubbed_registrations(),
        vec!["brew list --versions".to_string()],
        "the whole defect was that this was written and never read; it is read now"
    );
}

#[tokio::test]
async fn a_registered_command_is_not_recorded() {
    let m = mock();
    m.set_response("brew list --versions", Ok(DryRunOutput::new().into()));
    run(&m, "brew", &["list", "--versions"]).await;
    assert!(
        m.unstubbed_registrations().is_empty(),
        "a question the test answered is not an unanswered question"
    );
}

/// **The ledger is append-only, and that is deliberate.** Stating the assumption before the probe
/// means the probe is never recorded; asking first and registering afterwards leaves the record,
/// because the question *was* asked before the test had an answer for it. The alternative —
/// dropping entries once they become answered — would forgive precisely the case worth forgiving
/// least, and would make the ledger's contents depend on the order two unrelated calls happened
/// in. It is a record of what the run did, like the call log, and it is read as one.
#[tokio::test]
async fn an_existence_probe_is_recorded_too() {
    let stated = mock();
    stated.set_command_exists("btrfs", true);
    assert!(stated.check_command("btrfs"), "yes, as it said");
    assert!(
        stated.unstubbed_registrations().is_empty(),
        "an assumption stated before the question is not an unanswered question"
    );

    let silent = mock();
    assert!(
        silent.check_command("btrfs"),
        "the default is yes, by design: a mock answering `false` would have every backend report \
         itself unavailable"
    );
    assert_eq!(
        silent.unstubbed_registrations(),
        vec!["command -v btrfs".to_string()],
        "and the fact that the test never said so is on the record"
    );
}

/// The assertion itself. `Drop` panics, so the test that proves it works catches the unwind
/// rather than letting it fail the process — which is the only honest way to test a drop-time
/// assertion, since a passing version of it proves nothing on its own.
#[tokio::test]
async fn deny_unstubbed_turns_a_recorded_fall_through_into_a_failure() {
    let m = mock();
    m.deny_unstubbed();
    run(&m, "brew", &["list", "--versions"]).await;
    assert_eq!(
        m.unstubbed_registrations().len(),
        1,
        "the ledger still holds it"
    );

    let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || drop(m)));
    let err =
        dropped.expect_err("an unanswered command with `deny_unstubbed` set must fail at drop");
    let text = err
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| err.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        text.contains("brew list --versions"),
        "the failure must name the command, or it is a scolding: {text}"
    );
}

/// The negative, because a flag that can only ever fail is one nobody turns on by accident.
#[tokio::test]
async fn without_the_flag_the_same_fall_through_is_silent() {
    let m = mock();
    run(&m, "brew", &["list", "--versions"]).await;
    let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || drop(m)));
    assert!(
        dropped.is_ok(),
        "172 of the suite's tests leave a command unanswered, and reddening all of them is why \
         this is per-mock"
    );
}

/// The two flags are about different facts, and one must not be able to switch off the other.
#[tokio::test]
async fn allowing_a_dead_stub_does_not_also_allow_an_unanswered_call() {
    let m = mock();
    m.allow_unmatched_registrations();
    m.deny_unstubbed();
    m.set_response("never called", Ok(DryRunOutput::new().into()));
    run(&m, "brew", &["list", "--versions"]).await;
    let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || drop(m)));
    assert!(
        dropped.is_err(),
        "`allow_unmatched_registrations()` is about a stub that went unused; it says nothing \
         about a call that found no stub, and reading it as a blanket leniency would make one \
         flag mean two unrelated things"
    );
}
