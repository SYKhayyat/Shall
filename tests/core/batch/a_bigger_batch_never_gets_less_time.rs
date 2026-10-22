use super::*;

/// **Monotone, because a budget that shrinks as the work grows is not a budget.** The old
/// `clamp(1, 16)` was monotone too, so this is not the defect — it is the property the
/// narrowing must not break while fixing the cap.
#[test]
fn a_bigger_batch_never_gets_less_time() {
    let mut last = Duration::ZERO;
    for n in 1..=64 {
        let got = node_deadline(NODE, n, TOTAL);
        assert!(got >= last, "a batch of {n} got {got:?} after {last:?}");
        last = got;
    }
}
