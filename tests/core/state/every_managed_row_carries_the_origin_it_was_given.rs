use super::*;

/// Every writer supplies an origin, and it survives the round trip. The `add` signature
/// makes an absent one impossible; this pins that an *empty* one cannot sneak through the
/// same door, since `""` is a `&str` and the type cannot tell the difference.
#[test]
fn every_managed_row_carries_the_origin_it_was_given() {
    let mut r = reg();
    for source in ["modules/dev.txt:14", "adopt", "imperative", "hook:choco"] {
        r.add("apt", "jq", None, Default::default(), source, false);
        let row = r.get_package("apt", "jq").unwrap();
        assert_eq!(row.source, source);
        assert!(!row.source.is_empty());
    }
}
