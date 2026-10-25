use super::*;

/// `active` is committed and shared. On the desktop, `when host == laptop { Travel }` is
/// activating nothing — so there is nothing there to deactivate, and editing it would
/// change a machine nobody is sitting at.
#[test]
fn a_block_for_another_host_is_never_touched() {
    let e = deactivate(
        "when host == laptop {\n  Travel\n}\n",
        "desktop",
        &["Travel"],
    );
    assert!(e.removed.is_empty());
    assert!(!e.changed());
    assert_eq!(e.elsewhere.len(), 1);
    assert_eq!(e.elsewhere[0].predicate, "host == laptop");
    assert_eq!(e.elsewhere[0].line, 2);
    assert_eq!(e.body, "when host == laptop {\n  Travel\n}\n");
}
