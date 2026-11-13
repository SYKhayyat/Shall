//! An `invariant:` line says three things, and this reads them (#74).
//!
//! **The three are all required, and each refusal names the one that is missing.** A promise with
//! no path checks nothing, one with no target asks about no repository, and one with no deadline
//! is satisfied by a copy of anything from any year — the shape of a promise nobody checked.

use shall::config::grammar::{Options, Origin};
use shall::model::invariant::Invariant;

fn origin() -> Origin {
    Origin::new("modules/backup.txt", 1)
}

fn options(pairs: &[(&str, &str)]) -> Options {
    let mut out = Options::default();
    for (key, value) in pairs {
        out.set(*key, *value);
    }
    out
}

fn good() -> Options {
    options(&[
        ("path", "/srv/seforim"),
        ("backed-by", "restic:b2:seforim"),
        ("max-age", "7d"),
    ])
}

#[test]
fn a_complete_line_parses() {
    let inv = Invariant::parse(&origin(), "seforim", &good()).expect("valid");
    assert_eq!(inv.name, "seforim");
    assert_eq!(inv.path, "/srv/seforim");
    assert_eq!(inv.adapter, "restic");
    assert_eq!(inv.target, "b2:seforim");
    assert_eq!(inv.max_age.num_days(), 7);
}

#[test]
fn the_target_may_carry_its_own_colons() {
    // `b2:bucket:path` is one repository named by two more colons; splitting on the last one
    // would name the adapter `restic:b2:bucket`.
    let inv = Invariant::parse(
        &origin(),
        "x",
        &options(&[
            ("path", "/a"),
            ("backed-by", "restic:b2:bucket:path"),
            ("max-age", "1d"),
        ]),
    )
    .expect("valid");
    assert_eq!(inv.adapter, "restic");
    assert_eq!(inv.target, "b2:bucket:path");
}

#[test]
fn a_missing_option_names_the_one_that_is_missing() {
    let mut opts = good();
    opts.remove("max-age");
    let err = Invariant::parse(&origin(), "seforim", &opts).expect_err("refused");
    assert!(err.to_string().contains("missing `@max-age`"), "{err}");
}

#[test]
fn an_empty_option_is_refused_rather_than_read_as_absent() {
    let opts = options(&[("path", " "), ("backed-by", "restic:x"), ("max-age", "7d")]);
    let err = Invariant::parse(&origin(), "seforim", &opts).expect_err("refused");
    assert!(err.to_string().contains("empty `@path`"), "{err}");
}

#[test]
fn a_backed_by_without_a_target_is_refused() {
    let opts = options(&[
        ("path", "/srv/x"),
        ("backed-by", "restic"),
        ("max-age", "7d"),
    ]);
    let err = Invariant::parse(&origin(), "x", &opts).expect_err("refused");
    assert!(err.to_string().contains("does not say which tool"), "{err}");
}

#[test]
fn a_line_with_no_name_is_refused() {
    let err = Invariant::parse(&origin(), " ", &good()).expect_err("refused");
    assert!(err.to_string().contains("needs a name"), "{err}");
}

#[test]
fn the_units_are_the_three_it_says_and_nothing_else() {
    for (written, days) in [("7d", 7), ("36h", 1), ("90m", 0)] {
        let mut opts = good();
        opts.set("max-age", written);
        let inv = Invariant::parse(&origin(), "x", &opts).expect(written);
        assert_eq!(inv.max_age.num_days(), days, "{written}");
    }
    for written in ["7w", "7", "d", "0d", "-1d", "7 d"] {
        let mut opts = good();
        opts.set("max-age", written);
        assert!(
            Invariant::parse(&origin(), "x", &opts).is_err(),
            "`{written}` was accepted as a deadline"
        );
    }
}
