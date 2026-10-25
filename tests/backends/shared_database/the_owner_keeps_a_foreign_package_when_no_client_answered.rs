use super::*;

/// **The owner stands aside only when somebody can take its place.** On an Arch box with
/// no helper installed, `pacman:<aur package>` is still the best row there is — dropping
/// it would lose the package from the listing entirely, which is worse than a row whose
/// reinstall needs a tool the machine has not got.
#[test]
fn the_owner_keeps_a_foreign_package_when_no_client_answered() {
    let foreign = ForeignSets::of("pacman", &["shall-git"]);
    let kept: Vec<String> = one_row_per_shared_database(vec![pkg("pacman", "shall-git")], &foreign)
        .into_iter()
        .map(|p| format!("{}:{}", p.backend, p.name))
        .collect();
    assert_eq!(kept, vec!["pacman:shall-git"]);
}
