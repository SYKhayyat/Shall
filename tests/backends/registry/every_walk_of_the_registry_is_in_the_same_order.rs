use super::*;

/// Two walks of the registry give the same order, and it is one a reader can predict.
///
/// It was a `HashMap`, so the order was Rust's per-process hash seed: `shall list` printed
/// its backend blocks in a different sequence every run — two runs a second apart differed
/// by 530 lines and sorted identical — and the fan-outs handed their first slots to
/// whichever managers the seed happened to name first, so no timing measurement repeated.
///
/// Asserted against a *sorted copy*, not against a recorded list, so the test says "in an
/// order somebody can predict" rather than pinning today's set of backend names.
#[tokio::test]
async fn every_walk_of_the_registry_is_in_the_same_order() {
    let reg = build_registry().await;
    let names = |bs: Vec<std::sync::Arc<BackendCapabilities>>| {
        bs.iter().map(|b| b.name().to_string()).collect::<Vec<_>>()
    };

    let first = names(reg.all());
    assert_eq!(first, names(reg.all()), "two walks, two orders");
    let mut sorted = first.clone();
    sorted.sort();
    assert_eq!(first, sorted, "the order is not one a reader can predict");

    // `available()` filters the same walk, so it inherits the same guarantee — and it is
    // the one every listing command actually calls.
    let avail = names(reg.present_on_this_machine());
    let mut avail_sorted = avail.clone();
    avail_sorted.sort();
    assert_eq!(avail, avail_sorted);
}
