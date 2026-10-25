use super::*;

/// **A lock left in place says why.** The whole of `Q50`'s misdiagnosis came from the one
/// run where the answer was "no" printing nothing, so the reason had to be guessed from a
/// machine where the answer was "yes".
#[test]
fn a_lock_that_is_left_alone_is_still_reported() {
    let found = find(
        &Fake {
            alive: vec![],
            running: vec!["pacman"],
        },
        &reader(&[("/var/lib/pacman/db.lck", "")]),
    );
    assert!(found.stale.is_empty());
    assert_eq!(found.left.len(), 1, "{found:?}");
    assert!(
        found.left[0].because.contains("a `pacman` is running"),
        "{}",
        found.left[0].because
    );
}
