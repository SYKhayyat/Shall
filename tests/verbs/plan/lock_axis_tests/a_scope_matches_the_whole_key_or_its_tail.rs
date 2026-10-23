use super::*;

/// Both halves of a key are things a person types, on every axis. One rule, and the same
/// rule for version pins, bare names, hooks, adapters, `exec:`, events and generators —
/// so a scope cannot work on one ledger and quietly miss its twin.
#[test]
fn a_scope_matches_the_whole_key_or_its_tail() {
    for (key, tail) in [
        ("apt:curl", "curl"),
        ("after_install:nginx", "nginx"),
        ("adapters:backends.toml", "backends.toml"),
        ("exec:./setup.sh", "./setup.sh"),
        ("event:before_sync@repo", "before_sync@repo"),
        ("generate:./pick.sh", "./pick.sh"),
        (
            "health:systemctl is-active nginx",
            "systemctl is-active nginx",
        ),
    ] {
        assert!(scoped_by(key, &names(&[key])), "the whole key: {key}");
        assert!(scoped_by(key, &names(&[tail])), "the tail: {key}");
        assert!(!scoped_by(key, &names(&["something-else"])), "{key}");
    }
}
