use super::*;

#[test]
fn manage_three_delete_576_is_a_mistake_at_any_scale() {
    // II.11's example, and V.20's rule: a count cannot catch this on a small machine.
    assert!(reads_as_a_mistake(3, 576));
}
