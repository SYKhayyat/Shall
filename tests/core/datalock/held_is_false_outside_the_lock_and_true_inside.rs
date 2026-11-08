use super::*;

/// **`held_on` in both directions.** Every other test here asserts it is true inside a lock;
/// nothing asserted it is false outside one, so a `held_on -> true` survived the suite.
#[test]
fn held_is_false_outside_the_lock_and_true_inside() {
    let _g = gate();
    let dir = tmp("held-both-ways");
    assert!(
        !held_on(&dir),
        "nothing in this process holds the lock, and `held_on` says it does"
    );
    {
        let _taken = DataLock::acquire(&dir, "writer", Duration::from_secs(1)).unwrap();
        assert!(held_on(&dir), "this process is inside a lock it just took");
    }
    assert!(
        !held_on(&dir),
        "the lock was dropped and the directory was left recorded as held, so every later \
         `for_this_write` writes unlocked and reports success"
    );
}
