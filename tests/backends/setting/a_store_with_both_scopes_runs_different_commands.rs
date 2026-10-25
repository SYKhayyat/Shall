use super::*;

/// U19: a store with both scopes runs different commands for each. Without this the two
/// scopes would be a label on identical behaviour.
#[test]
fn a_store_with_both_scopes_runs_different_commands() {
    let a = registry_like();
    assert!(a.has_system_scope());
    let (_, user) = a.write_command(Scope::User, "Software/X", "k", "v");
    let (_, system) = a.write_command(Scope::System, "Software/X", "k", "v");
    assert_ne!(user, system, "system scope reused the per-user command");
    assert!(system.iter().any(|x| x.contains("HKLM")), "{:?}", system);
}
