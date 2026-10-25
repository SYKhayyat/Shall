use super::*;

/// The other half of B1: the counters are the *outcome*, so a caller cannot report the size
/// of its plan. They are only ever handed achieved work now, and nothing here invents any.
#[test]
fn the_counters_hold_only_what_they_were_given() {
    let m = MetricsCollector::new();
    assert_eq!(m.totals(), (0, 0, 0));
    m.record_install(2);
    m.record_remove(1);
    assert_eq!(m.totals(), (2, 1, 0));
}
