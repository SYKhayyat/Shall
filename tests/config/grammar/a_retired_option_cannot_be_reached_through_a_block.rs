use super::*;

#[test]
fn a_retired_option_cannot_be_reached_through_a_block() {
    // `@lease` is the one that mattered: II.16 retired it, and something downstream still
    // read it and turned it into a real expiry (S19).
    let err = doc("apt:jq {\n  lease = 2h\n}\n").unwrap_err().to_string();
    assert!(err.contains("lease"), "{}", err);
}
