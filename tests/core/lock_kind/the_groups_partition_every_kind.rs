use super::*;

/// The groups are exactly their parts, and the two of them are exactly everything. Asserted
/// rather than assumed, because a kind added to `ALL` and to neither group would be
/// unreachable by any group name and nobody would notice.
#[test]
fn the_groups_partition_every_kind() {
    let mut from_groups: Vec<LockKind> = PACKAGES.iter().chain(SCRIPTS.iter()).copied().collect();
    from_groups.sort();
    let mut all = ALL.to_vec();
    all.sort();
    assert_eq!(from_groups, all, "a kind belongs to no group, or to both");
    assert_eq!(ALL.len(), PACKAGES.len() + SCRIPTS.len());
}
