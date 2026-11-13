//! `check durability` (#74): is there still a copy of what the config promised, and how old is it?
//!
//! **The measurement is the backup tool's own listing, or it is nothing.** These tests drive the
//! listing and watch the verdict change — including to "nothing could answer", which is the one
//! answer that must never be reported as a pass. A check that read a receipt Shall had written
//! would agree with every one of these listings and with a repository that had been unreachable
//! for a month.

use chrono::{Duration, Utc};
use shall::app::durability::{self, Verdict};
use shall::config::grammar::{Options, Origin};
use shall::core::executor::DryRunOutput;
use shall::model::invariant::Invariant;

use crate::mock_providers::TestKernel;

const LISTING: &str = "restic -r b2:seforim snapshots --json --path /srv/seforim";

fn origin() -> Origin {
    Origin::new("modules/starter.txt", 1)
}

fn options(pairs: &[(&str, &str)]) -> Options {
    let mut out = Options::default();
    for (key, value) in pairs {
        out.set(*key, *value);
    }
    out
}

/// The line as a user writes it, with `backed_by` varied by the caller.
fn invariant(backed_by: &str) -> Invariant {
    Invariant::parse(
        &origin(),
        "seforim",
        &options(&[
            ("path", "/srv/seforim"),
            ("backed-by", backed_by),
            ("max-age", "7d"),
        ]),
    )
    .expect("the fixture's own line must parse")
}

/// What a listing says, in the shape `restic snapshots --json` prints it.
fn listing(times: &[chrono::DateTime<Utc>]) -> String {
    let rows: Vec<String> = times
        .iter()
        .map(|t| {
            format!(
                r#"{{"time":"{}","paths":["/srv/seforim"]}}"#,
                t.to_rfc3339()
            )
        })
        .collect();
    format!("[{}]", rows.join(","))
}

/// Measure one promise against `listing_body` — or against nothing at all, when the case is
/// that the machine never gets as far as asking.
///
/// `None` rather than an empty stub on purpose: a registered response that is never matched is
/// a test asserting against a command the product did not run (`#162`'s ledger), and the two
/// cases here are exactly that.
async fn measured(
    kernel: &TestKernel,
    listing_body: Option<&str>,
    backed_by: &str,
) -> Vec<durability::Answer> {
    if let Some(body) = listing_body {
        kernel.mock_executor.set_response(
            LISTING,
            Ok(DryRunOutput {
                stdout: body.as_bytes().to_vec(),
                stderr: Vec::new(),
            }
            .into()),
        );
    }
    let rows = shall::backends::backup::adapters(Vec::new());
    durability::measure(
        &kernel.app.executor,
        &rows,
        &[(invariant(backed_by), origin())],
    )
    .await
}

/// The ordinary case: the listing is stubbed and the product runs it.
async fn listed(kernel: &TestKernel, listing_body: &str) -> Vec<durability::Answer> {
    measured(kernel, Some(listing_body), "restic:b2:seforim").await
}

#[tokio::test]
async fn a_copy_younger_than_the_promise_holds_it() {
    let kernel = TestKernel::new().await;
    let answers = listed(&kernel, &listing(&[Utc::now() - Duration::hours(3)])).await;

    assert!(
        matches!(answers[0].verdict, Verdict::Fresh { .. }),
        "{:?}",
        answers[0].verdict
    );
    let finding = durability::finding(&answers);
    assert!(finding.ok, "{}", finding.summary);
    assert_eq!(finding.counts.get("fresh"), Some(&1));
}

#[tokio::test]
async fn a_copy_older_than_the_promise_breaks_it() {
    let kernel = TestKernel::new().await;
    let answers = listed(&kernel, &listing(&[Utc::now() - Duration::days(30)])).await;

    assert!(
        matches!(answers[0].verdict, Verdict::Stale { .. }),
        "{:?}",
        answers[0].verdict
    );
    let finding = durability::finding(&answers);
    assert!(!finding.ok);
    assert_eq!(finding.counts.get("stale"), Some(&1));
    assert!(
        finding.summary.contains("older than promised"),
        "{}",
        finding.summary
    );
}

#[tokio::test]
async fn the_newest_copy_is_the_one_that_counts() {
    let kernel = TestKernel::new().await;
    // A year-old snapshot beside yesterday's: the promise is about the newest copy, so listing
    // order must not decide it.
    let answers = listed(
        &kernel,
        &listing(&[
            Utc::now() - Duration::days(1),
            Utc::now() - Duration::days(365),
        ]),
    )
    .await;

    assert!(
        matches!(answers[0].verdict, Verdict::Fresh { .. }),
        "{:?}",
        answers[0].verdict
    );
}

#[tokio::test]
async fn an_empty_listing_is_its_own_verdict_and_not_a_failure_to_read() {
    let kernel = TestKernel::new().await;
    let answers = listed(&kernel, "[]").await;

    assert_eq!(answers[0].verdict, Verdict::Never);
    let finding = durability::finding(&answers);
    assert!(!finding.ok, "a path in no repository is not a kept promise");
    assert_eq!(finding.counts.get("never"), Some(&1));
}

#[tokio::test]
async fn a_tool_that_is_not_installed_is_unanswerable_and_never_a_pass() {
    let kernel = TestKernel::new().await;
    kernel.mock_executor.set_command_exists("restic", false);

    let answers = measured(&kernel, None, "restic:b2:seforim").await;

    assert!(
        matches!(answers[0].verdict, Verdict::Unanswerable(_)),
        "{:?}",
        answers[0].verdict
    );
    let finding = durability::finding(&answers);
    assert!(
        !finding.ok,
        "a promise nothing could check was reported as kept: {}",
        finding.summary
    );
    assert_eq!(finding.counts.get("unanswerable"), Some(&1));
}

#[tokio::test]
async fn an_adapter_nothing_ships_is_named_beside_the_ones_that_do() {
    let kernel = TestKernel::new().await;
    let answers = measured(&kernel, None, "kopia:nas").await;

    let Verdict::Unanswerable(why) = &answers[0].verdict else {
        panic!(
            "a missing adapter was not reported as unanswerable: {:?}",
            answers[0].verdict
        );
    };
    assert!(why.contains("`kopia`"), "{why}");
    assert!(
        why.contains("restic"),
        "the refusal must name the adapters that exist: {why}"
    );
}

/// **The listing decides, not anything Shall wrote.**
///
/// Same fixture, same line, one thing changed: what the tool answered. A check whose verdict
/// survived that would be reading its own bookkeeping.
#[tokio::test]
async fn the_answer_changes_only_because_the_listing_did() {
    let kernel = TestKernel::new().await;
    let fresh = listed(&kernel, &listing(&[Utc::now()])).await;
    let stale = listed(&kernel, &listing(&[Utc::now() - Duration::days(400)])).await;

    assert!(fresh[0].verdict.holds());
    assert!(!stale[0].verdict.holds());
}
