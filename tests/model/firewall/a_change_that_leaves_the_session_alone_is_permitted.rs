use super::*;

#[test]
fn a_change_that_leaves_the_session_alone_is_permitted() {
    assert_eq!(
        would_close_session(&[port(8080)], false, &[port(22)], Some(22)),
        None
    );
}
