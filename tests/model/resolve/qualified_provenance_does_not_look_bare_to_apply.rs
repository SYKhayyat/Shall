use super::*;

#[test]
fn qualified_provenance_does_not_look_bare_to_apply() {
    let options = Options::default();
    let origin = Origin::new("modules/base.txt", 1);
    let spec = to_spec(
        "apt",
        &Selector::Name("htop".into()),
        &options,
        true,
        None,
        Provenance {
            origin: &origin,
            scopes: &[],
            gates: &[],
            bare: false,
        },
    );
    assert_eq!(spec.options.one("__bare"), None);
}
