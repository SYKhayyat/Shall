use super::*;

#[test]
fn parse_log_skips_blank_lines() {
    assert!(parse_log("\n\n").is_empty());
}
