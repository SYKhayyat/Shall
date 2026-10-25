use super::*;

/// Which backends a bare `shall adopt` declines to take, and why it is a short list.
///
/// Opting out is a real cost to the user — a backend that keeps itself out of `adopt` is a
/// backend they have to write by hand — so it is spelled out here rather than left to
/// whoever adds the next one. The bar is not "this list is noisy": it is *being on the
/// machine is not evidence anybody chose it*, which is true of an init's running services
/// and of nothing else Shall drives.
///
/// Measured before the ruling: `adopt` wrote 161 declarations on a Windows host and 150
/// were services (owner ruling, 2026-08-05 — `Q39`).
#[tokio::test]
async fn only_the_backends_that_cannot_know_your_intent_opt_out_of_adopt() {
    let reg = build_registry().await;
    let available = reg.present_on_this_machine();
    let mut opted_out: Vec<&str> = available
        .iter()
        .filter(|b| b.as_queryable().is_some_and(|q| !q.adopted_unasked()))
        .map(|b| b.name())
        .collect();
    opted_out.sort();
    assert_eq!(
        opted_out,
        vec!["service"],
        "a backend joined or left the set `adopt` does not take unasked. Joining it means \
         users must write those lines by hand, so it needs the same argument `service` \
         has: an init reports what is running and never who chose it."
    );
}
