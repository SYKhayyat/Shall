use super::*;

/// The owner wins wherever it answered, and a listing filtered to a client is not emptied.
#[test]
fn a_shared_database_yields_one_holder_and_never_none() {
    let mut all = vec!["pacman".to_string(), "yay".into(), "paru".into()];
    one_backend_per_shared_database(&mut all);
    assert_eq!(all, vec!["pacman"]);

    // No owner in the list: the first client stands, so the answer is never empty.
    let mut clients = vec!["yay".to_string(), "paru".into()];
    one_backend_per_shared_database(&mut clients);
    assert_eq!(clients, vec!["yay"]);

    // Backends with databases of their own are all kept, in the caller's order.
    let mut unrelated = vec!["npm".to_string(), "apt".into(), "cargo".into()];
    one_backend_per_shared_database(&mut unrelated);
    assert_eq!(unrelated, vec!["npm", "apt", "cargo"]);
}
