use super::*;

#[test]
fn either_spelling_alone_resolves_to_that_release() {
    let from_bare = one_release("sharkdp/fd", "10.2.0", Some(release("10.2.0")), None).unwrap();
    assert_eq!(from_bare.version, "10.2.0");
    let from_prefixed =
        one_release("sharkdp/fd", "10.2.0", None, Some(release("v10.2.0"))).unwrap();
    assert_eq!(from_prefixed.version, "v10.2.0");
}
