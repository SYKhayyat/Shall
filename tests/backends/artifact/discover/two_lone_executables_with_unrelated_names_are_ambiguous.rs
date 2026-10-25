use super::*;

#[test]
fn two_lone_executables_with_unrelated_names_are_ambiguous() {
    let e = entries(&[("bin/rg", true), ("bin/rgx", true)]);
    assert!(matches!(
        find_executable(&e, "x/tool", None),
        Err(DiscoveryError::Ambiguous { .. })
    ));
}
