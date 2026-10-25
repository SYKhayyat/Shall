use super::*;

/// Neither direction is unreachable: `--steps` asks for them on a narrowed run, `--no-steps`
/// declines them on a whole-machine one. A default that cannot be overridden is a rule.
#[test]
fn the_flags_reach_both_answers_the_default_does_not_give() {
    let curl = named(&["curl"]);

    let default_narrow = req(&curl, None, false, None);
    assert!(!default_narrow.steps.unwrap_or(!default_narrow.narrowed()));

    let asked = req(&curl, None, false, Some(true));
    assert!(
        asked.steps.unwrap_or(!asked.narrowed()),
        "`--steps` must run them on a run that named a package"
    );

    let declined = req(&[], None, false, Some(false));
    assert!(
        !declined.steps.unwrap_or(!declined.narrowed()),
        "`--no-steps` must decline them on a whole-machine run"
    );
}
