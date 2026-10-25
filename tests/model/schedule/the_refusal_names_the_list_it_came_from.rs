use super::*;

/// The refusal names the list and its contents, so the way out is in the error rather than
/// in the documentation.
#[test]
fn the_refusal_names_the_list_it_came_from() {
    let o = opts(&[("cron", "0 2 * * *"), ("run", "rebuild")]);
    let err = schedule_config("nightly", &o, &origin(), &shipped())
        .unwrap_err()
        .to_string();
    assert!(err.contains("never_unattended"), "{}", err);
    assert!(err.contains("rebuild, purge-undeclared"), "{}", err);
}
