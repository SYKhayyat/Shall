use super::*;

/// The reported case. `pub` installs to `~/.pub-cache/bin`, and the whole finding is that
/// nothing mentioned it.
#[test]
fn a_per_user_ecosystem_names_its_bin_dir() {
    let dir = conventional_bin_dir("pub").expect("pub installs executables under $HOME");
    assert!(
        dir.ends_with("bin"),
        "a bin dir that is not a bin dir: {}",
        dir.display()
    );
}
