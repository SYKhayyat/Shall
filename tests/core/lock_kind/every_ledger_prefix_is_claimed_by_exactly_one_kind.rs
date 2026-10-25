use super::*;

/// Every ledger id maps to a kind, and the by-elimination case maps only what it should.
#[test]
fn every_ledger_prefix_is_claimed_by_exactly_one_kind() {
    for (id, expect) in [
        ("exec:./setup.sh", LockKind::Exec),
        ("generate:./pick.sh", LockKind::Generate),
        ("event:before_sync@repo", LockKind::Events),
        ("adapters:backends.toml", LockKind::Adapters),
        ("health:systemctl is-active nginx", LockKind::Health),
        ("vars:vars.shall", LockKind::Vars),
        ("after_install:nginx", LockKind::Hooks),
        ("before_install:redis", LockKind::Hooks),
        ("after_install:*", LockKind::Hooks),
    ] {
        assert_eq!(LockKind::of_ledger_id(id), expect, "{id}");
    }

    let mut prefixes: Vec<&str> = SCRIPTS.iter().filter_map(|k| k.ledger_prefix()).collect();
    prefixes.sort_unstable();
    let before = prefixes.len();
    prefixes.dedup();
    assert_eq!(before, prefixes.len(), "two kinds share a ledger prefix");
    assert_eq!(before, SCRIPTS.len() - 1, "only `hooks` may lack a prefix");
}
