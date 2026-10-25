use super::*;

/// **`init` writes the backup glob exactly once, and re-writes it when the user has commented
/// it out.** The behaviour was already there and is what this pins: a commented line is not a
/// pattern they have, so a second `init` puts it back — and a second `init` with it present
/// must not append a duplicate, because a `.gitignore` that grows on every run is a file the
/// user stops trusting.
///
/// **Written because the code above it changed for a reason that had nothing to do with
/// behaviour**: a lint that only the advisory job runs flagged the one-element loop, and the
/// rewrite it forced is exactly the kind of change that should arrive with a test rather than
/// on its own.
#[test]
fn init_writes_the_backup_glob_once_and_restores_a_commented_one() {
    // `init` refuses on a machine without git (X.5 keeps git optional rather than making its
    // absence an empty answer), so this is the module's usual skip rather than a special case.
    if !git_or_skip() {
        return;
    }
    let dir = tempdir().expect("a temp config root");
    let manager = GitManager::new(dir.path());
    let ignore = dir.path().join(".gitignore");

    manager.init().expect("first init writes the glob");
    let after_first = std::fs::read_to_string(&ignore).expect("a .gitignore");
    assert_eq!(
        after_first
            .lines()
            .filter(|l| l.trim() == "*.shall-backup")
            .count(),
        1,
        "the first init wrote the glob {after_first:?}"
    );

    manager.init().expect("second init is a no-op");
    assert_eq!(
        std::fs::read_to_string(&ignore).expect("a .gitignore"),
        after_first,
        "a second init appended to a .gitignore that already had the pattern"
    );

    std::fs::write(&ignore, "# *.shall-backup\n").expect("the user comments it out");
    manager.init().expect("third init restores it");
    let after_third = std::fs::read_to_string(&ignore).expect("a .gitignore");
    assert!(
        after_third.lines().any(|l| l.trim() == "*.shall-backup"),
        "a commented-out pattern is not a pattern the user has, so init must put it back: \
         {after_third:?}"
    );
    assert_eq!(
        after_third
            .lines()
            .filter(|l| l.trim() == "*.shall-backup")
            .count(),
        1,
        "the restored glob was written twice: {after_third:?}"
    );
}
