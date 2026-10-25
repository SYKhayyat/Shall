use super::*;

/// Every other adapter table refuses a second row claiming a name. This one kept it, so
/// which of two `vault` blocks answered depended on file order and nothing said so.
#[test]
fn a_second_provider_claiming_a_name_is_refused() {
    let kept = providers(vec![
        provider("vault", &["vault", "kv", "get", "{ref}"], true),
        provider("vault", &["something-else", "{ref}"], true),
    ]);
    assert_eq!(kept.len(), 1);
    assert_eq!(
        kept[0].decrypt[0], "vault",
        "the first claim on the name is the one that answers"
    );
}
