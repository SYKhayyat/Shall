use super::*;

/// A fraction is legal in every one of these tools (`lvcreate -L 1.5G`), so it is legal here.
#[test]
fn a_fractional_size_is_a_size() {
    assert_eq!(parse_size("1.5G"), Some(1536 * (1 << 20)));
    assert_eq!(parse_size("0.5M"), Some(512 * 1024));
}
