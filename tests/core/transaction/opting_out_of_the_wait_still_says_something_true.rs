use super::*;

/// `manager_lock_wait_secs = 0` opts out of waiting — and still does not print the old
/// sentence, because a further retry is exactly what *would* help once the holder is done.
#[test]
fn opting_out_of_the_wait_still_says_something_true() {
    let verdict = lock_wait_verdict(
        &lock_failure(PACMAN_SAID),
        "pacman",
        Duration::ZERO,
        Duration::ZERO,
        &|_| Held::Live("a `pacman`".into()),
    );
    let LockWait::Hopeless(err) = verdict else {
        panic!("with no wait allowed there is nothing to wait for: {verdict:?}");
    };
    let said = err.to_string();
    assert!(said.contains("manager_lock_wait_secs"), "{said}");
    assert!(
        !said.contains("a further retry will not help"),
        "the old sentence is the false one: {said}"
    );
}
