use super::*;

/// **A lock that could not be taken is one of two things, and the two are not the same
/// answer.**
///
/// The gate that found this: `cargo mutants` reported eight survivors in this file, and every
/// one of them was in the code that decides whether a second Shall is standing in the way.
/// `is_lock_contended -> true` made **every** error a contended lock — so a Shall that failed
/// for any reason at all (a bad path, a permission problem, a filesystem that does not lock)
/// concluded it was blocked by another Shall and waited, or gave up. **A bug that is not a lock
/// becomes a lock**, and a Shall that waits for a lock nobody holds waits for ever.
///
/// **Both directions, from the same fixture**, because a test that asserts only one of them is
/// what let the `-> true` and `-> false` mutants through.
#[test]
fn a_contended_lock_is_told_from_a_failure_that_is_not_one() {
    // **Contended.** `WouldBlock` is what the OS says when a lock is held elsewhere, and
    // `AlreadyExists` is the other name the same fact goes by.
    for kind in [
        std::io::ErrorKind::WouldBlock,
        std::io::ErrorKind::AlreadyExists,
    ] {
        let error = std::io::Error::from(kind);
        assert!(
            matches!(classify_lock_failure(error), LockFailure::Contended),
            "{kind:?} is the OS saying the lock is held elsewhere, so it is contention"
        );
    }

    // **And the raw code, because `std` maps it to the same kind** — on Linux `EWOULDBLOCK`
    // and `EAGAIN` are the same number, and `std` turns both into `WouldBlock`. This is the
    // branch the old code carried *as well*, and it was unreachable: the `kind()` arm above
    // had already answered every error it could have answered.
    #[cfg(unix)]
    for code in [libc::EWOULDBLOCK, libc::EAGAIN] {
        let error = std::io::Error::from_raw_os_error(code);
        assert!(
            matches!(classify_lock_failure(error), LockFailure::Contended),
            "the raw code {code} is contention whatever `std` calls it"
        );
    }

    // **Not contention.** These are the failures a Shall must not mistake for a busy machine.
    for kind in [
        std::io::ErrorKind::PermissionDenied,
        std::io::ErrorKind::NotFound,
        std::io::ErrorKind::Other,
    ] {
        let error = std::io::Error::from(kind);
        match classify_lock_failure(error) {
            LockFailure::Contended => panic!("{kind:?} is a failure, not a busy lock"),
            LockFailure::Other(e) => assert_eq!(
                e.kind(),
                kind,
                "the reason must survive the classification, because it is the answer the \\
                 caller reports"
            ),
        }
    }

    // **And the raw code for a failure, which is the case the `kind()` arm does not cover
    // and the raw-code arm used to.**
    #[cfg(unix)]
    {
        let error = std::io::Error::from_raw_os_error(libc::EACCES);
        match classify_lock_failure(error) {
            LockFailure::Contended => panic!("EACCES is a permission problem, not a busy lock"),
            LockFailure::Other(e) => assert_eq!(e.raw_os_error(), Some(libc::EACCES)),
        }
    }
}
