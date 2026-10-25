use super::*;

/// `flatpak install` calls an already-installed ref an error and exits non-zero — the
/// binary carries `Error: %s%s%s already installed`. Every sync that repairs a `@channel`
/// hands it a ref it may already have, so without `--or-update` the repair fails the
/// transaction over a machine that was already in the declared state.
#[test]
fn an_already_installed_ref_is_not_an_error_to_shall() {
    let argv = install_argv(&["--user"], &[spec_with("org.gimp.GIMP", &[])]);
    assert!(
        argv.contains(&"--or-update".to_string()),
        "install must tolerate a ref that is already there: {argv:?}"
    );
}
