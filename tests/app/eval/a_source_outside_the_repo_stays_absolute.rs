use super::*;

/// A path outside the repo is left absolute rather than mangled into a relative path that
/// would name a different file. `preferences.toml` on the machine is not in the repo.
#[test]
fn a_source_outside_the_repo_stays_absolute() {
    let out = repo_relative("/etc/shall/machine.toml:2", Path::new("/home/a/repo"));
    assert_eq!(out, "/etc/shall/machine.toml:2");
}
