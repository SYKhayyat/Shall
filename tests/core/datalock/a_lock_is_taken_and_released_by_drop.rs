use super::*;

#[test]
fn a_lock_is_taken_and_released_by_drop() {
    let _g = gate();
    let dir = tmp("release");
    {
        let _held = DataLock::acquire(&dir, "sync", Duration::from_secs(1)).unwrap();
    }
    // The same process can take it again once the first guard is gone.
    let _again = DataLock::acquire(&dir, "plan", Duration::from_secs(1)).unwrap();
}
