use super::*;

/// **`shall upgrade curl` is not a whole-machine upgrade**, so it does not run the
/// `@on=upgrade` steps. Every way of narrowing a run, because the one that caused this —
/// naming a package — was not what `scope()` was asking about, and a check written against
/// `scope()` alone would still fire firmware for `upgrade curl`.
#[test]
fn every_way_of_narrowing_a_run_stops_the_steps() {
    let curl = named(&["curl"]);
    assert!(req(&curl, None, false, None).narrowed(), "a named package");
    assert!(
        req(&[], Some("apt"), false, None).narrowed(),
        "a named manager"
    );
    assert!(req(&[], None, true, None).narrowed(), "--security");
}
