use super::*;

/// The manual filter decides from the source **path**, not the type string: the captured
/// fixture above proves real mise spells both global and project sources `mise.toml`, so
/// the old `== "global"` matched nothing and manual tools were invisible for ever.
#[test]
fn user_level_is_decided_by_where_the_source_lives() {
    let core = MiseBackendCore::new(CommandExecutor::new(false, false));
    let roots = [PathBuf::from("/root/.config/mise")];
    let pkg = |props: &[(&str, &str)]| {
        let mut p = Package::new("node", "mise");
        for (k, v) in props {
            p.properties.insert((*k).to_string(), (*v).to_string());
        }
        p
    };

    // Global config: user level, whatever its file is named.
    assert!(core.is_user_level_under(
        &roots,
        &pkg(&[
            ("source_type", "mise.toml"),
            ("source_path", "/root/.config/mise/config.toml")
        ])
    ));
    // A project checkout's pin is not Shall's to offer.
    assert!(!core.is_user_level_under(
        &roots,
        &pkg(&[
            ("source_type", "mise.toml"),
            ("source_path", "/srv/app/.mise.toml")
        ])
    ));
    // No source at all: installed ad hoc — manual by definition.
    assert!(core.is_user_level_under(&roots, &pkg(&[])));
}
