use super::*;

#[test]
fn the_default_reader_is_a_person() {
    // A struct built without naming the field must not silently promise a document: a
    // half-built `SyncOptions` that defaulted to `Json` would swallow every human line.
    assert!(Output::default().is_human());
}
