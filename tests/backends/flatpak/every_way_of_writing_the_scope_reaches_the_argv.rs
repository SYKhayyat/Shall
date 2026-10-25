use super::*;

/// Both spellings, the absent case, and the absent *block* — the argv difference is which
/// installation gets written, so a wrong answer here is a package installed for the wrong
/// people.
#[test]
fn every_way_of_writing_the_scope_reaches_the_argv() {
    for (written, expect, flag) in [
        (Some("user"), Scope::User, "--user"),
        (Some("system"), Scope::System, "--system"),
        (None, FLATPAK_DEFAULT_SCOPE, "--system"),
    ] {
        let block = written.map(|w| settings(&[("scope", w)]));
        let scope = FlatpakBackendCore::scope_from_settings(block.as_ref()).unwrap();
        assert_eq!(scope, expect, "scope = {written:?}");
        let core = FlatpakBackendCore::new(CommandExecutor::new(false, false), scope);
        assert_eq!(core.scope_args(), vec![flag]);
        assert_eq!(
            core.needs_root(),
            scope == Scope::System,
            "a user-scoped install writes under $HOME and must not ask for root"
        );
    }
    assert_eq!(
        FlatpakBackendCore::scope_from_settings(None).unwrap(),
        FLATPAK_DEFAULT_SCOPE,
        "no `[backend_settings.flatpak]` block at all"
    );
}
