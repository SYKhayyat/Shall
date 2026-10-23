use super::*;

/// A "nothing matched" warning has to say what it looked for, and the selection is half of
/// that. Both shapes, because a message that quoted only the names would leave a
/// class-scoped run explaining nothing.
#[test]
fn the_empty_scope_warning_describes_whichever_scope_was_given() {
    let apt = LockSelection::parse("versions:apt", &[]).unwrap();
    assert!(scope_description(&[], &apt).contains("versions:apt"));

    let both = scope_description(&names(&["curl"]), &apt);
    assert!(
        both.contains("versions:apt") && both.contains("curl"),
        "{both}"
    );

    let plain = scope_description(&names(&["curl"]), &LockSelection::everything());
    assert!(plain.contains("curl"), "{plain}");
}
