use super::*;

/// The owner's clarification: writing the scope that is already the default is accepted,
/// not refused as redundant. A config may state what it would also get for free.
#[test]
fn writing_the_default_scope_is_accepted() {
    let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
    let g = gsettings();
    assert_eq!(
        core.scope_of(&g, Some("user"), "org.gnome.x/k", None)
            .unwrap(),
        Scope::User
    );
    // And omitting it means the same thing.
    assert_eq!(
        core.scope_of(&g, None, "org.gnome.x/k", None).unwrap(),
        Scope::User
    );
}
