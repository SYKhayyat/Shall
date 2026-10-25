use super::*;

#[test]
fn the_two_classes_shall_controls_carry_a_budget_and_the_others_do_not() {
    assert!(Class::of("eval").budget().is_some());
    assert!(Class::of("info").budget().is_some());
    // A host with forty managers is slow because it has forty managers.
    assert!(Class::of("list").budget().is_none());
    assert!(Class::of("sync").budget().is_none());
}
