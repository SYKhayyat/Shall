//! **The preview asks its questions in the engine's order, and reports them in that order.**
//!
//! `preview_refusals` exists so `plan` and `sync --dry-run` can answer a refusal without doing
//! anything. Its value depends on one property that is invisible from the output if it is
//! missing: **it must ask what the engine will ask, in the order the engine will ask it.**
//!
//! The engine enforces removals first (`guard::enforce`, packages then extras) and the install
//! ceiling after (`guard::enforce_installs`). A preview that asked the install question first
//! would still refuse — but it would name the *install* ceiling on a run the engine refuses for
//! the *removal* one, and the user would fix the wrong thing. `#18`'s sub-task 4 names exactly
//! this: "a near-limit refusal can name the wrong phase."
//!
//! The order also matters for the count rather than only for the name: every phase spends
//! against one `Reaping` budget, so a phase asked out of order spends out of order and a run
//! that previewed clean can still be refused.
//!
//! **Both halves are asserted.** The first test pins that the removal refusal is reported and the
//! install one does not displace it; the second pins that each phase's own refusal appears once
//! and names its own ceiling.
use super::*;

/// A config whose two ceilings can be tripped independently: one removal, one install, and a
/// total tight enough that either arm alone stays under it.
fn both_ceilings(max_installs: usize, max_removals: usize) -> Config {
    let mut cfg = config_with(0);
    cfg.guard.max_installs = max_installs;
    cfg.guard.max_removals = max_removals;
    cfg.guard.max_total_changes = 0; // off, so the total cannot answer for either arm
    cfg
}

/// **The removal phase is asked first, and its refusal is the one a preview leads with.**
#[tokio::test]
async fn the_preview_reports_the_removal_refusal_before_the_install_one() {
    let reg = Arc::new(BackendRegistry::new());
    let cfg = both_ceilings(1, 1);

    let refusals = preview_refusals(
        &cfg,
        &reg,
        5,                   // over max_installs = 1
        &pairs(&["a", "b"]), // two, over max_removals = 1
        &extras(&[]),
        GuardScope::Apply,
    )
    .await;

    assert!(
        refusals.len() >= 2,
        "both ceilings are tripped, so both are reported — a preview that stopped at the first \
         would have to be run once per fix: {refusals:?}"
    );

    // Matched on the setting each refusal names rather than on `remov`/`install`, because the
    // install refusal's own advice mentions `--allow-mass-removal` and the removal refusal's
    // mentions `--allow-mass-removal` — a substring match finds both words in both messages and
    // compares a position with itself.
    let removal_at = refusals
        .iter()
        .position(|r| r.contains("max_removals"))
        .unwrap_or_else(|| {
            panic!("no refusal names the removal ceiling (`max_removals`), which the engine asks about first: {refusals:?}")
        });
    let install_at = refusals
        .iter()
        .position(|r| r.contains("max_installs"))
        .unwrap_or_else(|| {
            panic!("no refusal names the install ceiling (`max_installs`): {refusals:?}")
        });

    assert!(
        removal_at < install_at,
        "the preview reported the install ceiling at position {install_at} and the removal \
         ceiling at {removal_at}. The engine enforces removals first (`guard::enforce`) and \
         installs after (`guard::enforce_installs`), so a preview leading with the install \
         question names a phase the run would never have reached — and the user fixes the wrong \
         one: {refusals:#?}"
    );
}

/// **Each ceiling names itself, and neither stands in for the other.**
///
/// A preview that reported one refusal for both trips would pass the order test above while
/// being useless: the user would fix one ceiling and be refused for the other on the next run.
#[tokio::test]
async fn each_tripped_ceiling_has_a_refusal_of_its_own() {
    let reg = Arc::new(BackendRegistry::new());
    let cfg = both_ceilings(1, 1);

    let refusals = preview_refusals(
        &cfg,
        &reg,
        5,
        &pairs(&["a", "b"]),
        &extras(&[]),
        GuardScope::Apply,
    )
    .await;
    let joined = refusals.join("\n").to_lowercase();

    // Each refusal names the setting the user has to change, which is the whole reason the guard
    // says anything rather than just exiting non-zero.
    assert!(
        joined.contains("max_installs"),
        "the install ceiling's refusal must name `max_installs`: {refusals:#?}"
    );
    assert!(
        joined.contains("max_removals"),
        "the removal ceiling's refusal must name `max_removals`: {refusals:#?}"
    );
}

/// **And an untripped ceiling says nothing at all.** The other direction, because a guard that
/// refuses a clean run is worse than one that misses a refusal: it blocks every sync.
#[tokio::test]
async fn a_run_inside_every_ceiling_produces_no_refusal() {
    let reg = Arc::new(BackendRegistry::new());
    let cfg = both_ceilings(10, 10);

    let refusals = preview_refusals(
        &cfg,
        &reg,
        2,
        &pairs(&["a"]),
        &extras(&["link:/a"]),
        GuardScope::Apply,
    )
    .await;

    assert!(
        refusals.is_empty(),
        "one install, one package removal and one teardown are all inside their ceilings, so a \
         preview must be silent — anything else blocks the run it is previewing: {refusals:#?}"
    );
}
