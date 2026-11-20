use super::*;

/// Which backends a bare `shall adopt` declines to take, and why the list is now empty.
///
/// Opting out is a real cost to the user — a backend that keeps itself out of `adopt` is a
/// backend they have to write by hand — so it is spelled out here rather than left to whoever
/// adds the next one.
///
/// **It used to hold `service`, and it held it for a good reason that turned out to be the wrong
/// rule** (owner ruling `Q39`, 2026-08-05 → `C38`, 2026-11-20). Measured on a Windows host,
/// `adopt` wrote 161 declarations and 150 were services, two of them trigger-start services the
/// OS had already stopped again — a genuine observation, generalised into "a bare adopt takes
/// nothing an init merely reports". What it did not have was a narrowing to offer instead.
///
/// The narrowing now exists (`list_manual_enabled`, and the `plumbing` list of services the OS
/// authored), so the choice moved to where `II.99` puts every other choice: `adopt` offers
/// everything, and `--enabled-only` / `--all-services` are the flags that narrow it. A backend
/// that wants back in here has to argue that presence is not evidence of authorship *and* that
/// no narrowing is possible.
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
        Vec::<&str>::new(),
        "a backend joined the set `adopt` does not take unasked. Joining it means users must \
         write those lines by hand, so it needs an argument that no flag could ever narrow it."
    );
}
