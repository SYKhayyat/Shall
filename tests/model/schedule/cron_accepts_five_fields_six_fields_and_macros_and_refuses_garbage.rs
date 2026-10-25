use super::*;

#[test]
fn cron_accepts_five_fields_six_fields_and_macros_and_refuses_garbage() {
    for good in ["30 4 * * 1", "0 30 4 * * 1", "@daily", "0 2 * * *"] {
        assert!(validate_cron(good).is_ok(), "{} was refused", good);
    }
    assert!(validate_cron("not a cron").is_err());
}
