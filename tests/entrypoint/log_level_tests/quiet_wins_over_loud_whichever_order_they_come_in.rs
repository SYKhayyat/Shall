use super::*;

#[test]
fn quiet_wins_over_loud_whichever_order_they_come_in() {
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-q", "list"])),
        "error"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-q", "-vv", "list"])),
        "error"
    );
    assert_eq!(
        log_level_from_argv(&argv(&["shall", "-vv", "-q", "list"])),
        "error"
    );
}
