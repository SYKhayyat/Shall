use super::*;

/// **apt can be held but never stale.** The kernel releases an `flock` when its holder dies,
/// so a taken one has a live taker by definition — and the permanent file on disk says
/// nothing either way. Reporting `Stale` here would send a user to `heal` to delete a file
/// that must never be deleted.
#[test]
fn a_flock_is_held_only_while_something_is_running() {
    let files = reader(&[("/var/lib/dpkg/lock-frontend", "")]);
    assert_eq!(
        held_for(
            "apt",
            &Fake {
                alive: vec![],
                running: vec!["dpkg"]
            },
            &files
        ),
        Held::Live("a `dpkg`".into()),
    );
    assert_eq!(
        held_for(
            "apt",
            &Fake {
                alive: vec![],
                running: vec![]
            },
            &files
        ),
        Held::Free,
        "a permanent file with nothing running is not a stale lock",
    );
}
