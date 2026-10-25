use super::*;

#[test]
fn a_kind_with_no_account_domain_never_parses_a_suffix() {
    let row = "link:/home/alice/weird@user=bob";
    let key: ExtraKey = row.parse().expect("a path is a path");
    assert_eq!(key.account_suffix(), None);
    assert_eq!(key.subject, "/home/alice/weird@user=bob");
    assert_eq!(key.to_string(), row, "a path round-trips byte for byte");
}
