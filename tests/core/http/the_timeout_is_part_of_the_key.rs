use super::*;

#[test]
fn the_timeout_is_part_of_the_key() {
    let _bounded = client("shall-pool-c", false, 15).unwrap();
    let _unbounded = client("shall-pool-c", false, 0).unwrap();
    assert_eq!(mine("shall-pool-c"), 2);
}
