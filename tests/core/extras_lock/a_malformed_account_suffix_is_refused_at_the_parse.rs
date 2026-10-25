use super::*;

#[test]
fn a_malformed_account_suffix_is_refused_at_the_parse() {
    for bad in [
        "shim:rg@user=",
        "shim:@user=alice",
        "shim:rg@user=al ice",
        "shim:rg@user=a:b",
        "shim:rg@user=a@b",
        "shim:rg@user=a\nb",
        "service:nginx@user=",
    ] {
        assert_eq!(
            bad.parse::<ExtraKey>(),
            Err(()),
            "`{bad}` was read as an identity it does not have"
        );
    }
}
