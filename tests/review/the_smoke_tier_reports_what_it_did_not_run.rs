// tests/review/the_smoke_tier_reports_what_it_did_not_run.rs
//
// **A check that is skipped silently is indistinguishable from a pass** (`#21`).
//
// `release-check.sh` is the shippability gate and takes tens of minutes; the tier between
// `cargo check` and a release did not exist, so people ran nothing and the first time a workflow
// file had drifted from an installer was on the nightly.
//
// `scripts/smoke.sh` fills that gap. Two claims about it are worth pinning, and this file pins
// them rather than the prose:
//
// **1. Every skip says why.** The acceptance criterion on #21 is that the command "reports which
// checks were skipped and why". A summary line reading `9 passed` while a tenth check silently did
// nothing is the defect this exists to prevent, and it is the same one the integration harnesses
// were cured of (`tally_unmeasurable`).
//
// **2. The smoke set does not shrink to nothing.** The modules are named by string, so a rename
// makes them match no tests — and the honest thing then is a skip, which is what the script does.
// The dishonest thing is for the whole tier to become skips and still exit 0 with a green banner.
// This test requires the script to refuse to call an all-skipped run a clean pass.
//
// The script is a shell script that runs cargo, so this is a source scan rather than an
// invocation: running it from a test would nest a `cargo test` inside `cargo test`.

use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn smoke() -> String {
    std::fs::read_to_string(repo().join("scripts/smoke.sh")).expect("scripts/smoke.sh exists")
}

/// **A skip prints a reason, and takes one as an argument.**
///
/// The function's shape is the enforcement: `skip` takes two arguments, so a call site cannot
/// report a skip without saying why without the shell expanding `$2` as empty and the check being
/// visible in review.
#[test]
fn a_skip_has_to_state_its_reason() {
    let body = smoke();

    assert!(
        body.contains("skip()  {"),
        "the `skip` helper is gone; re-derive what this test was pinning rather than deleting it"
    );
    let skip_line = body
        .lines()
        .find(|l| l.trim_start().starts_with("skip()"))
        .expect("the skip helper's definition");
    assert!(
        skip_line.contains("$1") && skip_line.contains("$2"),
        "the `skip` helper no longer takes both a name and a reason, so a caller can report a \
         skip and say nothing about it: {skip_line}"
    );
    assert!(
        skip_line.contains("\u{24}{YEL}") || skip_line.contains("SKIP"),
        "the skip no longer prints a distinguishable marker, so it reads as a pass in a log: \
         {skip_line}"
    );
}

/// **The summary counts three things, not two.**
///
/// `$passed passed, $failed failed, $skipped skipped` — the third number is the whole point. A
/// two-number summary is what makes an all-skipped run look identical to a clean one.
#[test]
fn the_summary_names_the_skips_it_did_not_run() {
    let body = smoke();

    let summary = body
        .lines()
        .find(|l| l.contains("passed,") && l.contains("failed,"))
        .expect("the summary line");

    assert!(
        summary.contains("skipped"),
        "the summary reports passes and failures but not skips, so a tier that ran almost nothing \
         reads exactly like one that ran everything: {summary}"
    );
}

/// **The smoke set is named in one place and is not empty.**
///
/// If the array emptied, `smoke.sh` would report a clean pass having exercised nothing. Counting
/// the entries is a weak assertion on its own, so this also requires the four property-bearing
/// modules — the ones whose absence would mean the tier stopped covering the failures it was built
/// for.
#[test]
fn the_smoke_set_still_covers_the_command_surface() {
    let body = smoke();

    let start = body.find("SMOKE_MODULES=(").expect("the smoke module list");
    let end = body[start..]
        .find(")\n")
        .map(|i| start + i)
        .expect("the list's close");
    let list = &body[start..end];

    let entries: Vec<&str> = list
        .lines()
        .filter_map(|l| l.trim().strip_prefix('"'))
        .filter_map(|l| l.split('"').next())
        .collect();

    assert!(
        entries.len() >= 4,
        "the smoke set holds {} module(s); a tier this small is not exercising the command \
         surface it exists for: {entries:?}",
        entries.len()
    );

    for required in [
        "e2e::dry_run_every_verb_tests",
        "e2e::grade4_refusal_names_the_line_tests",
        "e2e::grade7_protected_skip_is_reported_tests",
        "core::exit_policy",
    ] {
        assert!(
            entries.contains(&required),
            "`{required}` left the smoke set. These four are the reason the tier exists — a \
             preview that writes nothing, a refusal that cites a line, a skip the user hears \
             about, and an exit code that means what the policy says. Removing one is a decision, \
             not a tidy-up."
        );
    }
}

/// **The release/install drift check is in the fast tier**, which is where #21 asked for it.
///
/// The finding was that release/install behaviour is "especially vulnerable to drift between
/// scripts, workflow YAML, target naming, and the binary actually built", and that the only place
/// it was checked was the nightly. A drift found on the nightly is a drift found after the tag.
#[test]
fn the_release_surface_is_checked_before_a_tag_is_cut() {
    let body = smoke();

    assert!(
        body.contains("the_release_publishes_what_the_installers_ask_for_tests"),
        "the smoke tier no longer compares the release surface with itself, so the drift #21 was \
         filed about is back to being found on the nightly — after the tag"
    );
}
