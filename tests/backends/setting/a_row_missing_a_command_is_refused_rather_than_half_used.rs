use super::*;

/// A row that cannot be read is not an adapter: X.4's read-before-write is what makes
/// `setting:` a declaration rather than a command that runs every sync.
#[test]
fn a_row_missing_a_command_is_refused_rather_than_half_used() {
    let mut unreadable = row("half", "halfctl");
    unreadable.read = vec![];
    assert!(!adapters(vec![unreadable]).iter().any(|a| a.name == "half"));

    let mut unresettable = row("half", "halfctl");
    unresettable.reset = vec![];
    assert!(!adapters(vec![unresettable])
        .iter()
        .any(|a| a.name == "half"));

    let mut nameless = row("", "halfctl");
    nameless.name = String::new();
    // Only the shipped rows remain once the nameless one is refused.
    assert_eq!(adapters(vec![nameless]).len(), 2);
}
