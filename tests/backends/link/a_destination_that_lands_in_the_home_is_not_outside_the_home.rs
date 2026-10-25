//! The same resolution answers the other direction, and it is a security question too: the
//! outside-home confirmation is what a user is asked before Shall writes outside their home, and
//! a destination written as `/tmp/x/.vimrc` reads as outside while landing inside.

use super::*;

#[cfg(unix)]
#[test]
fn a_destination_that_lands_in_the_home_is_not_outside_the_home() {
    let home = shall::core::account::Account::current()
        .expect("an account")
        .home()
        .to_path_buf();

    let written = tempdir().unwrap();
    let shortcut = written.path().join("looks-outside");
    std::os::unix::fs::symlink(&home, &shortcut).expect("the symlink");

    let target = shortcut.join(".vimrc");
    assert!(
        !target.starts_with(&home),
        "the test is not testing anything: the path reads as outside the home"
    );
    assert!(
        !is_outside_home(&target),
        "a destination that lands inside the home was reported as outside it, so the \
         confirmation was skipped for a write inside the user's own dotfiles"
    );

    // And an ordinary destination outside it is still outside, or the confirmation would be
    // asked about every file on the machine.
    assert!(is_outside_home(&written.path().join("config")));
}
