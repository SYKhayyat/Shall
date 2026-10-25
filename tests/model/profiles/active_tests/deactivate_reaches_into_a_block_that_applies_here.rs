use super::*;

/// The old rule — top-level lines only — left the name switched on by a block two lines
/// down and reported a state the machine had not reached. Reversed by owner 2026-07-17.
#[test]
fn deactivate_reaches_into_a_block_that_applies_here() {
    let e = deactivate(
        "Work\nwhen host == laptop {\n  Travel\n}\n",
        "laptop",
        &["Travel"],
    );
    assert_eq!(e.removed.len(), 1);
    assert_eq!(e.removed[0].gate.as_deref(), Some("host == laptop"));
    // The block held nothing else, so it goes too, and it says so.
    assert_eq!(e.emptied.len(), 1);
    assert_eq!(e.emptied[0].line, 2);
    assert_eq!(e.body, "Work\n");
    assert!(e.elsewhere.is_empty());
}
