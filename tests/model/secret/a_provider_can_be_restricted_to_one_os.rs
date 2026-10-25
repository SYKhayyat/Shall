use super::*;

/// The field this table was the only one of the seven not to have — so the one adapter
/// whose rows are handed a plaintext secret was the one that could not be confined to the
/// platform it was written for.
#[test]
fn a_provider_can_be_restricted_to_one_os() {
    let mut elsewhere = provider("winvault", &["cmdkey", "{ref}"], true);
    elsewhere.os = Some("definitely-not-this-os".into());
    assert!(!elsewhere.applies_to("linux"));
    assert!(!elsewhere.applies_to("windows"));
    assert!(
        providers(vec![elsewhere]).is_empty(),
        "a row for another platform is not a provider this machine can pick"
    );

    // A row that names no OS still applies everywhere, so no existing file changes meaning.
    assert_eq!(
        providers(vec![provider("vault", &["vault", "{ref}"], true)]).len(),
        1
    );
}
