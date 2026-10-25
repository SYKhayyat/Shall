use super::*;

/// `Mutating` carries no [`Shape`] because the heuristic is the wrong instrument for it, not
/// because it is unmeasured: [`scheduling_violation`] is its rule, and the engine feeds it.
/// A `Shape` there would fail a plan doing exactly what it was asked to — an edge is a wave
/// by design — which is why "give it a permissive one so it stops looking exempt" was the
/// wrong cure twice over.
#[test]
fn only_the_fan_out_class_carries_a_shape() {
    assert!(Class::of("list").shape().is_some());
    assert!(Class::of("check").shape().is_some());
    assert!(Class::of("sync").shape().is_none());
    assert!(Class::of("eval").shape().is_none());
    assert!(Class::of("info").shape().is_none());

    // And the class without a `Shape` still has a bound. Asserted here so a change that
    // deletes one instrument cannot leave the other reading as deliberate emptiness.
    assert!(
        scheduling_violation(6, 1, 6).is_some(),
        "`Mutating` has no shape budget and no scheduling rule either, which is the state \
         this pair was built to leave behind"
    );
}
