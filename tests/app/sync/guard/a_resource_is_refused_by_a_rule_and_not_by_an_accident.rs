use super::*;

/// `purge-undeclared` sweeps everything Shall does not manage, and it builds its list from
/// `list_installed` â€” which for `service` is every running service. The only thing that
/// ever stopped it was the declarability test asking a question about *package* lines and
/// getting the right answer for the wrong reason. Correcting that sentence would have
/// handed the sweep 155 Windows services; this is the refusal made on purpose instead.
#[test]
fn a_resource_is_refused_by_a_rule_and_not_by_an_accident() {
    let cfg = Config {
        // Even the escape hatch, wide open. A `service:` is not a package the user could
        // be declaring they manage themselves; there is nothing here to release.
        guard: shall::config::GuardSettings {
            unprotected_packages: vec!["*".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let empty = HashSet::new();

    for (backend, name) in [
        ("service", "AppMgmt"),
        ("service", "sshd"),
        ("link", "/home/u/.vimrc"),
        ("setting", "org.gnome.desktop.interface/clock-format"),
    ] {
        let p = protection_of(&cfg, Some(backend), name, &empty);
        assert!(
            matches!(p, Some(Protection::NotAPackage(_))),
            "`{backend}:{name}` must be refused as a resource, not swept: {p:?}"
        );
        let reason = p.unwrap().reason();
        assert!(
            !reason.contains("no line can hold"),
            "`{backend}:{name}` parses â€” saying no line can hold it is false: {reason}"
        );
    }

    // And the refusal is about resources, not about everything: a package is still judged
    // by the ordinary rules, or the guard has stopped being about the user's intent.
    assert!(protection_of(&cfg, Some("apt"), "jq", &empty).is_none());
}
