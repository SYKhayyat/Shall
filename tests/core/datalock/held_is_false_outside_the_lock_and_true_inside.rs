use super::*;

/// **`held()` in both directions.** Every other test here asserts it is true inside a lock;
/// nothing asserted it is false outside one, so `held() -> true` and the same function with
/// its `> 0` read as `>= 0` â€” which is every unsigned count â€” both survived the suite.
#[test]
fn held_is_false_outside_the_lock_and_true_inside() {
    let _g = gate();
    let dir = tmp("held-both-ways");
    assert!(
        !held(),
        "nothing in this process holds the lock, and `held()` says it does"
    );
    {
        let _taken = DataLock::acquire(&dir, "writer", Duration::from_secs(1)).unwrap();
        assert!(held(), "this process is inside a lock it just took");
    }
    assert!(
        !held(),
        "the lock was dropped and the count did not come back down, so every later `for_this_write` writes unlocked and reports success"
    );
}
