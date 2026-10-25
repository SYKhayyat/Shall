use super::*;

/// Every kind's name round-trips, and no two share one. A duplicate would make `expand`
/// return whichever came first in the array and silently shadow the other.
#[test]
fn every_kind_has_one_name_and_no_two_share_it() {
    for kind in ALL {
        assert_eq!(kinds_of(&sel(kind.as_str())), vec![kind], "{kind}");
    }
    let mut names: Vec<&str> = ALL.iter().map(|k| k.as_str()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "two kinds share a name");
}
