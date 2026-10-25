use super::*;

/// **The bug that cost a green CI run, as a unit test.** `is_zombie` reads
/// `/proc/<pid>/stat`, whose third field is the state letter — after the last `)`, because
/// a process name may itself contain spaces and parentheses.
///
/// A killed `pacman` keeps its `/proc` entry, and `comm` still says `pacman`, until its
/// parent reaps it. `heal` runs inside that window: it asked "is a pacman running", a corpse
/// said yes, and the lock stayed. The scan that found no pacman ran nine seconds later,
/// which made a race look like a contradiction.
#[test]
fn a_zombie_is_read_out_of_stat_whatever_the_process_was_called() {
    let dir = tempfile::TempDir::new().unwrap();
    let write = |body: &str| {
        std::fs::write(dir.path().join("stat"), body).unwrap();
        is_zombie(dir.path())
    };
    assert!(write("4242 (pacman) Z 1 4242 4242 0 -1 4194560"));
    assert!(!write("4242 (pacman) S 1 4242 4242 0 -1 4194560"));
    assert!(!write("4242 (pacman) R 1 4242 4242 0 -1 4194560"));
    // The name is the reason the field is found from the right: split from the left and
    // `(pac man)` moves every field along by one.
    assert!(write("4242 (pac man) Z 1 4242"));
    assert!(!write("4242 (pac man) S 1 4242"));
    // A stat nobody can read proves nothing, and "proves nothing" must not read as "dead":
    // that would clear a lock on the strength of a failed read.
    assert!(!is_zombie(std::path::Path::new("/no/such/proc/entry")));
}
