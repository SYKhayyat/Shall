use super::*;

/// The table must not name one backend twice: the second row would silently replace the
/// first in a reader's mind while both ran, and a contradiction between them would show up
/// as a flake rather than a failure.
#[test]
fn no_backend_has_two_argv_rows() {
    let mut seen: Vec<&str> = argv_cases().iter().map(|c| c.backend).collect();
    let before = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(
        before,
        seen.len(),
        "a backend has two rows in the argv table"
    );
}
