//! **A leftover sentence is owed only by a run that did not revert and did not fail.**
//!
//! `warn_about_leftovers` names the launcher a manager's own uninstall left on disk (`#136`,
//! C11). It runs at the end of `SyncEngine::sync`, guarded by two conditions, and **both of them
//! are about not saying it**:
//!
//! * a **revert** put the package back, so a file still on disk is the state the user asked for;
//! * a run that **carried past a failure** has already reported something louder, and the
//!   leftovers question is about what succeeded.
//!
//! Written inline, the guard was `!reverted && matches!(result, Ok([]))`, and `cargo mutants`
//! found a survivor on it: `&&` → `||` makes the warning fire after a revert and after a carried
//! failure, and **every existing test ran a clean successful removal**, where both spellings
//! agree. So the wrong sentence would have appeared only on the runs a user is already having
//! trouble with — the one place a misleading line costs the most.
//!
//! The predicate is now a function with its own name and its own tests, which is the smallest
//! thing that makes both arms assertable without standing up a snapshot provider and a failing
//! health check to reach them through the engine.
use super::*;
use shall::core::Error;

/// **A clean, successful, un-reverted run owes the sentence.** The one case that fires.
#[test]
fn a_run_that_removed_and_did_not_revert_owes_the_sentence() {
    assert!(
        owes_a_leftover_sentence(false, Ok(&[])),
        "a removal that stood and carried nothing past is exactly when a leftover is worth \
         naming — this is the run the whole warning exists for"
    );
}

/// **A revert owes nothing**, because the package is back.
#[test]
fn a_reverted_run_owes_nothing() {
    assert!(
        !owes_a_leftover_sentence(true, Ok(&[])),
        "the snapshot was restored, so the package is present again — a launcher on disk is the \
         state the user asked for, and naming it would accuse the machine of a leftover it does \
         not have"
    );
}

/// **A run that carried past a failure owes nothing**, because it already said something louder.
#[test]
fn a_run_that_carried_past_a_failure_owes_nothing() {
    let carried = [CarriedPast {
        name: "apt:libfoo".to_string(),
        retry: Retryability::Permanent,
        refused: false,
    }];
    assert!(
        !owes_a_leftover_sentence(false, Ok(&carried)),
        "four failures and a sentence about a launcher move the reader's attention off the \
         failures, which is the opposite of what this run needs"
    );
}

/// **And a run whose removals all failed outright owes nothing** — there is no leftover to name,
/// because nothing was removed. This is the `Err` arm, which the inline `matches!` never looked
/// at: a whole-run error carries no removal that stood.
#[test]
fn a_run_that_failed_outright_owes_nothing() {
    let e = Error::Other("the whole transaction was refused".to_string());
    assert!(
        !owes_a_leftover_sentence(false, Err(&e)),
        "a run that failed before removing anything has no leftover, and a sentence about one \
         would be about a file this run did not touch"
    );
}

/// **A revert outranks a clean result.** Both conditions have to hold, and this asserts the pair
/// rather than either one alone: with `reverted` true the answer is no whatever the result says,
/// which is the half of the `&&` a reader of the caller cannot see.
#[test]
fn a_revert_outranks_a_clean_result() {
    let e = Error::Other("x".to_string());
    for carried in [Ok(&[][..]), Err(&e)] {
        assert!(
            !owes_a_leftover_sentence(true, carried),
            "a revert restored the package, so no result makes a leftover sentence right"
        );
    }
}
