use super::*;

/// The three spellings each tool prints, and the bare byte count every one of them is read
/// as. `G` is 1024-based on all three; a decimal reading here would put every declared volume
/// 7% away from the one that got created.
#[test]
fn one_size_written_every_way_parses_to_one_number() {
    for spelling in [
        "10G",
        "10g",
        "10GB",
        "10gb",
        "10GiB",
        "10gib",
        "10737418240",
    ] {
        assert_eq!(
            parse_size(spelling),
            Some(10 * (1 << 30)),
            "{} is 10 GiB",
            spelling
        );
    }
    assert_eq!(parse_size("100M"), Some(100 * (1 << 20)));
    assert_eq!(parse_size("512K"), Some(512 * 1024));
    assert_eq!(parse_size("2T"), Some(2 * (1 << 40)));
    assert_eq!(parse_size("1P"), Some(1 << 50));
    assert_eq!(parse_size("4096"), Some(4096));
    assert_eq!(parse_size("4096B"), Some(4096));
    assert_eq!(parse_size(" 8G "), Some(8 * (1 << 30)));
}
