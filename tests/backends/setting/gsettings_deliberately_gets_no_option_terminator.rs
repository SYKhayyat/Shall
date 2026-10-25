use super::*;

/// `gsettings` dispatches on argv[1] by hand, so a `--` is read as the command name and
/// the call fails before it reaches the schema.
#[test]
fn gsettings_deliberately_gets_no_option_terminator() {
    assert!(!shall::core::argv::terminates_options("gsettings"));
    let a = gsettings();
    for (_, args) in [
        a.read_command(Scope::User, "org.gnome.x", "k"),
        a.write_command(Scope::User, "org.gnome.x", "k", "v"),
        a.reset_command(Scope::User, "org.gnome.x", "k"),
    ] {
        assert!(!args.iter().any(|x| x == "--"), "{:?}", args);
    }
}
