use super::*;

#[test]
fn add_line_writes_a_block_and_refuses_a_duplicate_name() {
    let body = add_line("", "nightly", "0 2 * * *", "sync", None).unwrap();
    assert!(body.contains("schedule:nightly {"));
    assert!(body.contains("cron = 0 2 * * *"));
    assert!(add_line(&body, "nightly", "0 3 * * *", "clean", None).is_err());
}
