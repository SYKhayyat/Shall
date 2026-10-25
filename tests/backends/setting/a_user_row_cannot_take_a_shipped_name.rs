use super::*;

#[test]
fn a_user_row_cannot_take_a_shipped_name() {
    let all = adapters(vec![row("gsettings", "impostor")]);
    let g = all.iter().find(|a| a.name == "gsettings").unwrap();
    assert_eq!(g.detect, "gsettings", "a user row shadowed the shipped one");
    // The impostor is rejected, so only the shipped rows remain.
    assert_eq!(all.len(), 2);
}
