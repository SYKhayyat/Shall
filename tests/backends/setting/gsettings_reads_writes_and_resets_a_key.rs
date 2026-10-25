use super::*;

#[test]
fn gsettings_reads_writes_and_resets_a_key() {
    let a = gsettings();
    let (p, args) = a.read_command(Scope::User, "org.gnome.x", "k");
    assert_eq!(p, "gsettings");
    assert_eq!(args, vec!["get", "org.gnome.x", "k"]);

    let (_, args) = a.write_command(Scope::User, "org.gnome.x", "k", "v");
    assert_eq!(args, vec!["set", "org.gnome.x", "k", "v"]);

    let (_, args) = a.reset_command(Scope::User, "org.gnome.x", "k");
    assert_eq!(args, vec!["reset", "org.gnome.x", "k"]);
}
