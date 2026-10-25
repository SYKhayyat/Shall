use super::*;

/// **Every key this ledger can write names a kind the teardown can dispatch on.**
///
/// The teardown is exhaustive over `ResourceKind` now, so the one way a row can still
/// arrive un-actionable is for the *key* to open with something that is not a keyword. That
/// is what a `&str` dispatch could never notice: it matched the arms it knew and shrugged
/// at the rest, and the shrug reported the undo as done.
#[test]
fn every_ledger_key_names_a_kind_the_teardown_can_dispatch_on() {
    let o = Options::default;
    let statements = [
        Statement::Shim("rg".into(), o()),
        Statement::Service("nginx".into(), o()),
        Statement::Setting("dark".into(), o()),
        Statement::Link("src".into(), o()),
        Statement::Schedule("nightly".into(), o()),
        Statement::Firewall("22/tcp".into(), o()),
        Statement::Repo {
            backend: "apt".into(),
            spec: "ppa:x/y".into(),
        },
    ];
    for stmt in &statements {
        let key = extra_key(stmt)
            .unwrap()
            .unwrap_or_else(|| panic!("{stmt:?} produced no key"));
        let parsed: ExtraKey = key
            .to_string()
            .parse()
            .unwrap_or_else(|_| panic!("`{key}` does not read back as a key"));
        assert_eq!(parsed, key, "`{key}` does not survive its own round trip");
        assert_eq!(
            Some(key.kind),
            stmt.kind(),
            "`{key}` names a different kind than the statement it came from"
        );
    }

    // And the three that must NOT be keyed: a verb has no inverse, and a tree's rows are
    // the `link:` keys its files were placed under.
    assert!(extra_key(&Statement::Exec("./x.sh".into(), o()))
        .unwrap()
        .is_none());
    assert!(extra_key(&Statement::Generate("./x.sh".into(), o()))
        .unwrap()
        .is_none());
    assert!(extra_key(&Statement::Dotfiles("tree".into(), o()))
        .unwrap()
        .is_none());
}
