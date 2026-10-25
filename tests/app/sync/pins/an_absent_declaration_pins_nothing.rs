use super::*;

/// `absent:jq@version=1.7` pins nothing to install — it says the package must not be there,
/// and a version on it cannot fail to be honoured by a removal.
#[test]
fn an_absent_declaration_pins_nothing() {
    let reg = registry(&[("cannot", false, true)]);
    let mut s = spec("jq", "cannot", Some("1.7"));
    s.present = false;
    assert!(found(&reg, &[s]).is_empty());
}
