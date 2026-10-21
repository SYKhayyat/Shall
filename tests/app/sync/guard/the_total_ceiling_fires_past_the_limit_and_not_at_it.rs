use super::*;

/// The same boundary, asked of the function the boundary lives in.
///
/// [`the_removal_ceiling_fires_past_the_limit_and_not_at_it`] was written to kill
/// `too_many_changes`'s `>`â†’`>=`, and did not: it sets `max_removals`, which is a
/// *per-kind* ceiling checked elsewhere, so `too_many_changes` â€” which reads
/// `max_total_changes` and nothing else â€” never saw a total equal to its limit. The mutant
/// was still alive nine days later. A boundary test has to be pointed at the comparison it
/// names, not at a ceiling that sounds like it.
#[test]
fn the_total_ceiling_fires_past_the_limit_and_not_at_it() {
    let cfg = |limit| Config {
        guard: shall::config::GuardSettings {
            max_total_changes: limit,
            ..Default::default()
        },
        ..Default::default()
    };

    assert!(
        too_many_changes(&cfg(3), &Reaping::new(), 3).is_none(),
        "three changes under a total of three is at the limit, not over it"
    );
    assert!(
        too_many_changes(&cfg(3), &Reaping::new(), 4).is_some(),
        "four changes under a total of three must be refused"
    );
    // The other half of the same guard: nought is off, not a ceiling of nought.
    assert!(
        too_many_changes(&cfg(0), &Reaping::new(), 9_999).is_none(),
        "max_total_changes = 0 disables the total; it does not refuse everything"
    );
    // And a limit of one, where `>` and `>=` disagree about the smallest set there is.
    assert!(too_many_changes(&cfg(1), &Reaping::new(), 1).is_none());
    assert!(too_many_changes(&cfg(1), &Reaping::new(), 2).is_some());
}
