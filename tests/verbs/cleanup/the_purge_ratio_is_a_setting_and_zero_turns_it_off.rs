use super::*;

/// The threshold is the owner's to move, in both directions.
///
/// **`0.0` is off, and that is a deliberate escape hatch rather than an accident of the
/// arithmetic** — `managed / to_remove < 0.0` is false for every non-negative input, so a
/// bare comparison would have disabled the rule at zero by luck. It is written down and
/// tested because someone purging a machine on purpose, repeatedly, should be able to say
/// so once in a file instead of remembering a flag every time.
#[test]
fn the_purge_ratio_is_a_setting_and_zero_turns_it_off() {
    let strict = Config {
        guard: shall::config::GuardSettings {
            purge_ratio: 0.5,
            ..Default::default()
        },
        ..Default::default()
    };
    // Passes at the shipped 0.1, refused once the owner asks for half.
    assert!(!reads_as_a_mistake(43, 276));
    assert!(shall::verbs::cleanup::reads_as_a_mistake(&strict, 43, 276));

    let off = Config {
        guard: shall::config::GuardSettings {
            purge_ratio: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(
        !shall::verbs::cleanup::reads_as_a_mistake(&off, 0, 576),
        "zero is off — managing nothing and deleting everything is allowed on request"
    );
}
