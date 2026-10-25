use super::*;

/// The limit is found by its header, not by counting columns: `btrfs-progs` has changed what
/// it prints across versions, and an index would read `excl` on the version that added one.
#[test]
fn the_qgroup_limit_is_read_by_name_and_none_is_not_a_number() {
    let with_limit = "qgroupid         rfer         excl     max_rfer \n\
                      --------         ----         ----     -------- \n\
                      0/257            16384        16384    10737418240\n";
    assert_eq!(qgroup_limit_in(with_limit), Some(10737418240));

    // `none` is btrfs saying there is no limit. It is not a size, and a declared `@quota=`
    // compared against it would be satisfied by a word.
    let unlimited = "qgroupid         rfer         excl     max_rfer \n\
                     --------         ----         ----     -------- \n\
                     0/257            16384        16384    none\n";
    assert_eq!(qgroup_limit_in(unlimited), None);

    // A column order this code did not choose. The header moves, the answer does not.
    let reordered = "qgroupid     max_rfer         rfer\n\
                     --------     --------         ----\n\
                     0/257        5368709120       16384\n";
    assert_eq!(qgroup_limit_in(reordered), Some(5368709120));

    // Quotas disabled: btrfs says so on stderr and prints no table at all.
    assert_eq!(qgroup_limit_in(""), None);
    assert_eq!(
        qgroup_limit_in("ERROR: can't list qgroups: quotas not enabled\n"),
        None
    );
}
