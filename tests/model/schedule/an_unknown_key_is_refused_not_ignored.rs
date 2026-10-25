use super::*;

#[test]
fn an_unknown_key_is_refused_not_ignored() {
    // A typo like `crron =` must not leave the job silently unscheduled.
    let o = opts(&[("crron", "0 2 * * *"), ("run", "clean")]);
    let err = schedule_config("t", &o, &origin(), &shipped()).unwrap_err();
    assert!(err.what.contains("unknown option `crron`"), "{}", err);
}
