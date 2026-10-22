use super::*;

/// The re-entrancy that stops the process waiting for itself.
///
/// `flock` is per open file description: a second handle opened by a process that already
/// holds the lock blocks until the first is released, which is never, because the code that
/// would release it is waiting.
#[test]
fn a_process_inside_the_lock_does_not_take_it_again() {
    let _g = gate();
    let dir = tmp("reentrant");
    let outer = DataLock::acquire(&dir, "outer", Duration::from_secs(1)).unwrap();
    assert!(held(), "this process is inside a lock it took");

    let inner = DataLock::for_this_write("a ledger").unwrap();
    assert!(
        inner.is_none(),
        "taking it a second time in one process is how this deadlocks"
    );

    drop(outer);
}
