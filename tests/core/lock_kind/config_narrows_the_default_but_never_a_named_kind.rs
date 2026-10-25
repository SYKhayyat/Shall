use super::*;

/// A standing preference narrows the default and never an explicit request. Both halves:
/// the point of the second is that `shall lock exec` must freeze `exec` on a machine whose
/// preferences leave it out — a command that declines what it was told to do is worse than
/// no preference at all.
#[test]
fn config_narrows_the_default_but_never_a_named_kind() {
    let allowed = [LockKind::Exec, LockKind::Hooks];

    let defaulted = LockSelection::everything().narrowed_by_config(&allowed);
    assert_eq!(kinds_of(&defaulted), vec![LockKind::Hooks, LockKind::Exec]);
    assert!(!defaulted.includes(LockKind::Versions));

    let asked = sel("versions").narrowed_by_config(&allowed);
    assert!(
        asked.includes(LockKind::Versions),
        "naming a kind is asking for it, whatever preferences.toml prefers"
    );

    let spelled = sel("everything").narrowed_by_config(&allowed);
    assert!(!spelled.includes(LockKind::Versions));
}
