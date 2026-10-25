use super::*;

/// `J3`'s ruling. An AUR package is in pacman's database and in no sync repository, so
/// pacman can remove it and cannot put it back. The row a user can act on — delete the
/// line, sync, put it back — is the helper's.
#[test]
fn a_package_the_owner_cannot_reinstall_is_named_under_the_client_that_can() {
    let foreign = ForeignSets::of("pacman", &["shall-git"]);
    let rows = vec![
        pkg("pacman", "bash"),
        pkg("pacman", "shall-git"),
        pkg("yay", "bash"),
        pkg("yay", "shall-git"),
        pkg("paru", "bash"),
        pkg("paru", "shall-git"),
    ];
    let kept: Vec<String> = one_row_per_shared_database(rows, &foreign)
        .into_iter()
        .map(|p| format!("{}:{}", p.backend, p.name))
        .collect();
    // A repository package still names pacman; only the foreign one moves, and it moves
    // to exactly one helper rather than to both.
    assert_eq!(kept, vec!["pacman:bash", "yay:shall-git"]);
}
