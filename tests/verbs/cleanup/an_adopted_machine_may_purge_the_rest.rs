use super::*;

#[test]
fn an_adopted_machine_may_purge_the_rest() {
    // Ubuntu after `adopt`: ~103 manual packages managed, the dependency closure and
    // whatever else is lying around unmanaged. That is the command working as intended.
    assert!(!reads_as_a_mistake(103, 476));
}
