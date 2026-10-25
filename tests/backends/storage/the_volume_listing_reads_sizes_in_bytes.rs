use super::*;

/// The listing carries each volume's size in bytes, so `sync` can see a `@size=` that no
/// longer matches (Q19). Bytes, because `lvs` would otherwise print `10.00g` and the
/// comparison would be against a rounded display string.
#[test]
fn the_volume_listing_reads_sizes_in_bytes() {
    let pkgs = parse_lvs_list(
        "  vg0 data 10737418240\n\
         \x20 vg0 logs 5368709120\n\
         \x20 vg1 data 1073741824\n\
         \n",
    );
    assert_eq!(pkgs.len(), 3, "a blank line is not a volume");
    let at = |n: &str| {
        pkgs.iter()
            .find(|p| p.name == n)
            .unwrap_or_else(|| panic!("{} was not listed", n))
            .properties
            .get("size")
            .map(String::as_str)
    };
    // `vg0/data` and `vg1/data` are two volumes; the group is part of the name, not decoration.
    assert_eq!(at("vg0/data"), Some("10737418240"));
    assert_eq!(at("vg0/logs"), Some("5368709120"));
    assert_eq!(at("vg1/data"), Some("1073741824"));
}
