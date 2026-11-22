//! C42's bound, stated as a test: the manager table is keyed by a home-relative **directory**, and
//! a checkout is reported as one finding however many files a `git clone` left inside it.

use std::fs;
use std::path::PathBuf;

use shall::app::dotfiles::shell_managers;

/// A scratch home outside the repository — a directory under `target/` is inside a git working
/// tree, and the scan's other rule would then answer for the fixture (the reason the link tests
/// build their own root too).
fn home(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("shall-shell-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn the_checkout_is_found_and_the_files_inside_it_are_one_finding() {
    let home = home("managers");
    fs::create_dir_all(home.join(".oh-my-zsh/plugins/git")).unwrap();
    for i in 0..50 {
        fs::write(home.join(format!(".oh-my-zsh/plugins/git/f{}", i)), "x").unwrap();
    }
    fs::create_dir_all(home.join(".nvm/versions/node/v20/bin")).unwrap();

    let found = shell_managers(&home);
    let names: Vec<&str> = found.iter().map(|(m, _)| m.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["nvm", "oh-my-zsh"],
        "one finding per checkout, sorted — fifty files inside one are still one line"
    );
    assert_eq!(
        found[1].1,
        home.join(".oh-my-zsh"),
        "the path reported is the directory, not something inside it"
    );
    assert_eq!(found[1].0.kind, "framework");
    assert_eq!(found[0].0.kind, "version-manager");

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn a_manager_that_is_not_there_is_not_reported() {
    let home = home("empty");
    assert!(
        shell_managers(&home).is_empty(),
        "a machine with none of these has nothing to leave alone"
    );
    let _ = fs::remove_dir_all(&home);
}
