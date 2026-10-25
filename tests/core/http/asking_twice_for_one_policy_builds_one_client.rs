use super::*;

#[test]
fn asking_twice_for_one_policy_builds_one_client() {
    for _ in 0..20 {
        let _ = client("shall-pool-a", false, 15).unwrap();
    }
    assert_eq!(
        mine("shall-pool-a"),
        1,
        "twenty asks for one policy built more than one client — the pool is not pooling"
    );
}
