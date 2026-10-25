use super::*;

/// The marker match is what triggers the wait at all, so every row's phrasing is checked
/// against the manager's real words — and against a failure that is *not* about a lock,
/// because a wait on every failed install is a hang on every typo.
#[test]
fn each_managers_own_words_for_a_taken_lock_are_recognised() {
    let cases: &[(&str, &str)] = &[
        (
            "pacman",
            "error: failed to init transaction (unable to lock database)",
        ),
        (
            "dnf",
            "Waiting for process with pid 1234 to finish. Another app is currently holding \
             the yum lock",
        ),
        (
            "zypper",
            "System management is locked by the application with pid 999",
        ),
        (
            "apt",
            "E: Could not get lock /var/lib/dpkg/lock-frontend. It is held by process 42",
        ),
    ];
    for (backend, said) in cases {
        assert!(
            says_the_lock_is_taken(backend, said),
            "{backend} did not recognise its own lock message: {said}"
        );
        assert!(
            !says_the_lock_is_taken(backend, "E: Unable to locate package qqqq"),
            "{backend} called a missing package a taken lock"
        );
    }
}
