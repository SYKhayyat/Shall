use super::*;

/// An unqualified kind admits every entry in it — the control without which every
/// assertion above passes against a matcher that admits nothing.
#[test]
fn an_unqualified_kind_admits_all_of_its_entries() {
    let all_versions = sel("versions");
    for key in ["apt:curl", "cargo:ripgrep", "brew:jq"] {
        assert!(all_versions.admits(LockKind::Versions, key, None), "{key}");
    }
    let all_hooks = sel("hooks");
    for key in ["after_install:nginx", "before_install:*"] {
        assert!(all_hooks.admits(LockKind::Hooks, key, None), "{key}");
    }
}
