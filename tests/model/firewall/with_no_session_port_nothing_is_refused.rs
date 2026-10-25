use super::*;

/// On a console there is no connection to lose, so nothing is refused on these grounds.
#[test]
fn with_no_session_port_nothing_is_refused() {
    assert_eq!(would_close_session(&[port(22)], true, &[], None), None);
}
