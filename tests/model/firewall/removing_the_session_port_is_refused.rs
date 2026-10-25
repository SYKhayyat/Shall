use super::*;

/// The plain case: removing the rule that holds this connection open.
#[test]
fn removing_the_session_port_is_refused() {
    assert_eq!(
        would_close_session(&[port(22)], false, &[], Some(22)),
        Some(22)
    );
}
