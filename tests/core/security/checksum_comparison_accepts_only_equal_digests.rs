use super::*;

#[test]
fn checksum_comparison_accepts_only_equal_digests() {
    assert!(constant_time_hex_eq("abcdef", "abcdef"));
    assert!(constant_time_hex_eq("abcdef", "ABCDEF"));
    assert!(!constant_time_hex_eq("abcdef", "abcdee"));
    assert!(!constant_time_hex_eq("abcdef", "abcdef0"));
}
