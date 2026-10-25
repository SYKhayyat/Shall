use super::*;

/// The three ways a host can answer with something that is not a check report. Each has to
/// be an error and none may read as "in sync" — the answer a fleet must never invent.
#[test]
fn an_answer_that_is_not_a_check_report_is_an_error() {
    for (what, json) in [
        ("not JSON at all", "shall: unrecognized subcommand 'status'"),
        ("JSON, but not an array of sections", r#"{"to_install":[]}"#),
        (
            "an array with no drift section",
            r#"[{"section":"health","ok":true}]"#,
        ),
    ] {
        assert!(
            parse_check(json).is_err(),
            "{} was accepted as a drift report",
            what
        );
    }
}
