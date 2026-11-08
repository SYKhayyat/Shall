use super::*;

/// The re-entrancy that stops the process waiting for itself.
///
/// `flock` is per open file description: a second handle opened by a process that already
/// holds the lock blocks until the first is released, which is never, because the code that
/// would release it is waiting. The question is asked of the directory the lock covers, so the
/// test points the write door at the directory it holds.
#[test]
fn a_process_inside_the_lock_does_not_take_it_again() {
    let _g = gate();
    let dir = tmp("reentrant");
    let _data_dir = DataDir::at(&dir);
    let outer = DataLock::acquire(&dir, "outer", Duration::from_secs(1)).unwrap();
    assert!(held_on(&dir), "this process is inside a lock it took");

    let inner = DataLock::for_this_write("a ledger").unwrap();
    assert!(
        inner.is_none(),
        "taking it a second time in one process is how this deadlocks"
    );

    drop(outer);
}
