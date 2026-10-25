use super::*;

#[test]
fn providers_drops_the_unusable_and_keeps_the_rest() {
    let rows = vec![
        provider("good", &["sops", "-d", "{ref}"], true),
        provider("leaky", &["writes-a-file", "{ref}"], false), // no stdout_only → dropped
    ];
    let kept = providers(rows);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].name, "good");
}
