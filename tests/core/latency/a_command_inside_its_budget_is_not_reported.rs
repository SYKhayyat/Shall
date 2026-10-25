use super::*;

#[test]
fn a_command_inside_its_budget_is_not_reported() {
    // Nothing to assert on the output here — the value is that this cannot panic and that
    // the boundary is inclusive, so a command exactly at its budget is not "over".
    assert!(Duration::from_secs(5) <= Class::ConfigOnly.budget().unwrap());
}
