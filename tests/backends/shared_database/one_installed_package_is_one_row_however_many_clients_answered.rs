use super::*;

#[test]
fn one_installed_package_is_one_row_however_many_clients_answered() {
    let rows = vec![
        pkg("pacman", "jq"),
        pkg("pacman", "bash"),
        pkg("yay", "jq"),
        pkg("yay", "bash"),
        pkg("paru", "jq"),
        pkg("paru", "bash"),
        pkg("npm", "jq"),
    ];
    let kept: Vec<String> = one_row_per_shared_database(rows, &ForeignSets::default())
        .into_iter()
        .map(|p| format!("{}:{}", p.backend, p.name))
        .collect();
    // `npm:jq` survives: a global npm package named jq is a different install, and
    // removing the pacman one leaves it. Sharing a *name* is not sharing a database.
    assert_eq!(kept, vec!["pacman:jq", "pacman:bash", "npm:jq"]);
}
