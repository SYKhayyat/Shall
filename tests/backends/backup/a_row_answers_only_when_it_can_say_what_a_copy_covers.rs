//! A backup adapter row answers one question: **what is the newest copy of this path in that
//! target?** (#74)
//!
//! The row is data, so the whole answer is testable without a repository: the argv is filled from
//! the line, and the listing is parsed. What the tests hold to is the split — a listing that
//! parses and holds nothing is "never backed up", and a listing that cannot be read is an error
//! that says so. Merging those two is how a check reports a path as unmeasured on the strength of
//! a repository nobody looked inside.

use chrono::{DateTime, Utc};
use shall::backends::backup::{self, BackupAdapter};
use shall::core::adapter::AdapterRow;

fn row() -> BackupAdapter {
    backup::adapters(Vec::new())
        .into_iter()
        .next()
        .expect("this build ships one row")
}

#[test]
fn the_shipped_row_is_restic_and_it_can_do_all_three_things() {
    let rows = backup::adapters(Vec::new());
    assert_eq!(
        rows.len(),
        1,
        "a new row needs a case here and a line in the docs"
    );
    assert_eq!(rows[0].name, "restic");
    assert_eq!(rows[0].unusable(), None);
}

#[test]
fn the_query_fills_the_path_and_the_target() {
    let q = row().query("/srv/seforim", "b2:seforim");
    assert_eq!(
        q,
        [
            "restic",
            "-r",
            "b2:seforim",
            "snapshots",
            "--json",
            "--path",
            "/srv/seforim"
        ]
    );
}

#[test]
fn the_newest_snapshot_is_the_answer() {
    let listing = r#"[
        {"time":"2026-06-22T03:14:15Z","paths":["/srv/seforim"]},
        {"time":"2026-06-21T03:10:00Z","paths":["/srv/seforim"]}
    ]"#;
    let newest = row().newest(listing).expect("valid").expect("a snapshot");
    assert_eq!(
        newest,
        DateTime::parse_from_rfc3339("2026-06-22T03:14:15Z")
            .unwrap()
            .with_timezone(&Utc)
    );
}

#[test]
fn an_empty_listing_is_never_backed_up_and_not_a_failure() {
    assert_eq!(row().newest("[]").expect("parses"), None);
}

#[test]
fn a_listing_that_is_not_json_is_a_named_error() {
    let err = row()
        .newest("Fatal: unable to open config file")
        .expect_err("refused");
    assert!(err.contains("not JSON"), "{err}");
}

#[test]
fn a_row_that_cannot_list_or_cannot_say_when_is_unusable() {
    let mut cannot_list = row();
    cannot_list.list = Vec::new();
    assert!(cannot_list.unusable().is_some());

    let mut no_time_field = row();
    no_time_field.time_field = String::new();
    assert!(no_time_field.unusable().is_some());
}

#[test]
fn a_named_adapter_is_found_case_insensitively_or_not_at_all() {
    let rows = backup::adapters(Vec::new());
    assert!(backup::by_name(&rows, "RESTIC").is_some());
    assert!(backup::by_name(&rows, "borg").is_none());
}
