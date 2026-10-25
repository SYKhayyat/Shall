use super::*;

/// The subtle case, and the reason this function takes the default policy at all: tightening
/// `default/incoming` closes the session's port without ever naming it.
#[test]
fn tightening_the_default_closes_the_session_port_without_naming_it() {
    assert_eq!(would_close_session(&[], true, &[], Some(22)), Some(22));
    // ...unless something still holds it open.
    assert_eq!(would_close_session(&[], true, &[port(22)], Some(22)), None);
}
