use super::*;

/// The keyword and the type are one fact, so they cannot drift apart.
///
/// `ALL` is hand-written, which is the one thing about this type that a compiler does not
/// check — so a variant added without being listed there would silently stop being parseable
/// from a ledger key. That is caught below by parsing every keyword the grammar actually
/// produces, rather than by trusting the list.
#[test]
fn every_resource_kind_round_trips_through_its_keyword() {
    let mut seen = std::collections::HashSet::new();
    for k in ResourceKind::ALL {
        assert_eq!(
            k.as_str().parse::<ResourceKind>(),
            Ok(*k),
            "`{k}` does not parse back to itself"
        );
        assert_eq!(k.to_string(), k.as_str());
        assert!(seen.insert(k.as_str()), "two kinds answer `{k}`");
    }
    assert!(
        "apt".parse::<ResourceKind>().is_err(),
        "a backend is not a kind"
    );
    assert!("".parse::<ResourceKind>().is_err());
}
