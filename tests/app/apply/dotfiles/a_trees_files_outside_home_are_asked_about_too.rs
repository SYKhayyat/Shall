use super::*;

/// SEC3 asks about a tree's files too, because a tree is the `link:` lines it stands for.
///
/// It could not before: the question was asked of `state.extras`, and a `dotfiles:` tree is
/// one statement there whose files are not. So `link:` into `/etc` asked and
/// `dotfiles:./etc @target=/etc` — the same destinations, forty at a time — did not.
#[test]
fn a_trees_files_outside_home_are_asked_about_too() {
    #[cfg(windows)]
    let root = std::path::PathBuf::from(r"C:\ProgramData\shall");
    #[cfg(not(windows))]
    let root = std::path::PathBuf::from("/etc/shall");

    // What `links_of` produces for a tree: one synthesised `link:` per placed file.
    let expanded = Dotfiles::links_of(&[Tree {
        name: "./etc".to_string(),
        origin: Origin::new("modules/files.txt", 1),
        plan: shall::model::dotfiles::TreePlan {
            placements: vec![shall::model::dotfiles::Placement {
                source: std::path::PathBuf::from("./etc/hosts"),
                destination: root.join("hosts"),
                relative: std::path::PathBuf::from("hosts"),
            }],
            collisions: Vec::new(),
        },
    }]);

    let asked = Dotfiles::outside_home_links(expanded.iter(), &|_| false).unwrap();
    assert_eq!(
        asked.len(),
        1,
        "a tree file landing outside home was not asked about"
    );
    assert_eq!(asked[0].1, root.join("hosts"));

    // And the same rule as a hand-written line: an existing destination was agreed to on
    // the run that placed it, so it is not asked about again.
    assert!(Dotfiles::outside_home_links(expanded.iter(), &|_| true)
        .unwrap()
        .is_empty());
}
