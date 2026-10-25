use super::*;

/// An `absent:` line carrying `@hold` is not a hold: it declares the package must NOT be
/// there, and freezing something against an upgrade it is not going to get is meaningless.
#[test]
fn an_absent_declaration_holds_nothing() {
    let mut s = spec("npm", "typescript", true);
    s.present = false;
    let holds = Holds::new(&StateRegistry::default(), Some(&desired(vec![s])), None);
    assert!(holds.is_empty());
}
