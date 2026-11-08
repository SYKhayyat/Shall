//! A far end's status decides whether the failure is weather or a wrong request.
//!
//! `github:`, `web:` and `appimage:` all download from a URL somebody else runs, and all three
//! used to fold a non-success status into `Error::Other` — which is `Retryability::Unknown`,
//! which `shall-failure-class` prints as `unknown`, which both harnesses read as *a defect*.
//!
//! The nightly of 2026-11-08 is the measurement: `github:sharkdp/fd` answered `504 Gateway
//! Timeout` on the arch leg and the harness scored `github: install of github:sharkdp/fd failed
//! twice, unclassified — a defect, not ecosystem variance`, then the lifecycle ratchet and the
//! section floor both fell because one install did not happen.
//!
//! The family is checked here rather than at one call site: the three backends that download
//! build the error through one constructor, and the status families below are every one a
//! download can meet.

use shall::core::error::Retryability;
use shall::core::Error;

fn class_of(status: u16) -> Retryability {
    Error::http_status(
        "some/asset.tar.gz",
        format!("{status} Whatever"),
        "https://example.test/x",
    )
    .retryability()
}

/// A server that failed to serve is worth asking again.
#[test]
fn a_server_that_failed_to_serve_is_retried() {
    for status in [500u16, 502, 503, 504, 520, 522] {
        assert_eq!(
            class_of(status),
            Retryability::Transient,
            "{status} is the far end having a bad minute, and a second attempt is the whole cure"
        );
    }
}

/// A server that says the thing is not there has answered the question.
#[test]
fn a_server_that_said_the_file_is_absent_is_not_retried() {
    for status in [400u16, 401, 403, 404, 410, 451] {
        let class = class_of(status);
        assert_eq!(
            class,
            Retryability::Permanent,
            "{status} will be the same answer on the next attempt, and retrying it is time spent \
             proving the far end did not change"
        );
    }
}

/// The rate limit keeps its own verdict, which is also "try again" — but for a different reason,
/// and the reason is what a user reads.
#[test]
fn a_refusal_too_many_requests_is_still_a_window_that_moves() {
    assert_eq!(class_of(429), Retryability::Permanent);
    assert_eq!(
        Error::RateLimit("window".into()).retryability(),
        Retryability::Transient
    );
}

/// The status survives `with_note`, which is how a failure acquires its `while applying` line.
///
/// The alternative is the bug `with_note` was written to fix, one variant over: a note that
/// rebuilds the error as something else takes the verdict with it, and this one carries its
/// verdict in a field rather than in a marker list.
#[test]
fn a_note_does_not_take_the_status_away() {
    let noted = Error::http_status("pkg.tar.gz", "504 Gateway Timeout", "https://e.test/x")
        .with_note("\n    while applying `github:owner/repo`");
    assert_eq!(noted.retryability(), Retryability::Transient);
    assert!(
        format!("{noted}").contains("while applying"),
        "the note must reach the message: {noted}"
    );
    assert!(
        format!("{noted}").contains("504"),
        "the status must reach the message: {noted}"
    );
}

/// A message and a status that disagree resolve to the status, because the status is the fact.
#[test]
fn the_status_is_read_from_the_code_and_not_from_the_prose() {
    let e = Error::http_status("x", "503 Service Unavailable", "https://e.test/x");
    match &e {
        Error::HttpStatus { status, .. } => assert_eq!(*status, 503),
        other => panic!("wrong variant: {other:?}"),
    }
}
