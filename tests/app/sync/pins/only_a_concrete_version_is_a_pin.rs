use super::*;

/// `latest` and `*` are the absence of a pin written out loud, and every manager honours
/// them by doing what it was going to do anyway. Refusing those would refuse the ordinary
/// case — and `@version=` parsing to an empty string is `AU10`, which is caught in the
/// grammar and must not turn into a refusal here.
#[test]
fn only_a_concrete_version_is_a_pin() {
    let reg = registry(&[("cannot", false, true)]);
    for loose in ["latest", "*", ""] {
        assert!(
            found(&reg, &[spec("jq", "cannot", Some(loose))]).is_empty(),
            "`{loose}` was read as a pin"
        );
    }
    assert!(found(&reg, &[spec("jq", "cannot", None)]).is_empty());
}
