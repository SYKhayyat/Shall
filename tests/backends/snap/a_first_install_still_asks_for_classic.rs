use super::*;

/// The install argv still carries `@classic` for a snap that is not there yet: Q20 is about
/// the *change*, and a first install must not regress into a strict one.
#[test]
fn a_first_install_still_asks_for_classic() {
    let mut spec = PackageSpec {
        name: "code".into(),
        backend: "snap".into(),
        options: Default::default(),
        requires: vec![],
        present: true,
    };
    spec.options.set("classic", "true");
    assert_eq!(
        install_args(&spec),
        vec!["install", "--classic", "--", "code"]
    );
    spec.options.set("classic", "false");
    assert_eq!(install_args(&spec), vec!["install", "--", "code"]);
}
