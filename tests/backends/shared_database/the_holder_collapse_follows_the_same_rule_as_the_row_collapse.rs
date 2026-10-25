use super::*;

/// The same rule on the holder collapse, which is what `--absent` writes its permanent
/// line from. The two paths answer one question and must not answer it differently.
#[test]
fn the_holder_collapse_follows_the_same_rule_as_the_row_collapse() {
    let foreign = ForeignSets::of("pacman", &["shall-git"]);

    let mut holders = vec!["pacman".to_string(), "yay".into(), "paru".into()];
    one_backend_for(&mut holders, "shall-git", &foreign);
    assert_eq!(holders, vec!["yay"]);

    // A repository package keeps the owner.
    let mut holders = vec!["pacman".to_string(), "yay".into(), "paru".into()];
    one_backend_for(&mut holders, "bash", &foreign);
    assert_eq!(holders, vec!["pacman"]);

    // Nobody to stand in: the owner keeps it.
    let mut alone = vec!["pacman".to_string()];
    one_backend_for(&mut alone, "shall-git", &foreign);
    assert_eq!(alone, vec!["pacman"]);
}
