use super::*;

/// An empty list refuses nothing. Stated as a test because the alternative — a list whose
/// emptiness silently restores the built-in pair — is the shape that makes a guard setting
/// unable to mean what it says.
#[test]
fn an_empty_list_refuses_nothing() {
    let o = opts(&[("cron", "0 2 * * *"), ("run", "rebuild")]);
    assert!(schedule_config("nightly", &o, &origin(), &[]).is_ok());
}
