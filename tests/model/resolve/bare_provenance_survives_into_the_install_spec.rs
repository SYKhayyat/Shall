use super::*;

#[test]
fn bare_provenance_survives_into_the_install_spec() {
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
            bare: true,
        },
    );
    assert_eq!(spec.options.one("__bare"), Some("true"));
}
