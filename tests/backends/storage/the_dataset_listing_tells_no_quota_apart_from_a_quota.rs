use super::*;

/// A quota is three states — a limit, a dataset read with no limit, and (by the property
/// being absent) a listing that could not run. The middle one is why `NO_LIMIT` exists: a
/// declared `@quota=` against a dataset with none is drift, and reading "no limit" as
/// "unknown" would leave that quota unapplied for ever.
#[test]
fn the_dataset_listing_tells_no_quota_apart_from_a_quota() {
    let pkgs = parse_zfs_list(
        "tank/capped\t10737418240\t/mnt/capped\n\
         tank/free\t0\t/mnt/free\n\
         tank/legacy\t0\tlegacy\n",
    );
    let at = |n: &str| {
        pkgs.iter()
            .find(|p| p.name == n)
            .unwrap()
            .properties
            .get("quota")
            .map(String::as_str)
    };
    assert_eq!(at("tank/capped"), Some("10737418240"));
    assert_eq!(at("tank/free"), Some(NO_LIMIT));
    assert_eq!(at("tank/legacy"), Some(NO_LIMIT));
    // The mountpoint column still reads correctly with a quota column in front of it.
    let mount = |n: &str| {
        pkgs.iter()
            .find(|p| p.name == n)
            .unwrap()
            .properties
            .get("mount")
            .map(String::as_str)
    };
    assert_eq!(mount("tank/capped"), Some("/mnt/capped"));
    assert_eq!(mount("tank/legacy"), None);
}
