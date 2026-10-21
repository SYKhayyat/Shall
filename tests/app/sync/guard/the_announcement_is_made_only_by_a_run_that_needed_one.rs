use super::*;

/// A run that needed no override says nothing, and one that did names what it overrode.
#[test]
fn the_announcement_is_made_only_by_a_run_that_needed_one() {
    let mut cfg = config_with(10);
    cfg.allow_mass_removal = true;
    assert_eq!(
        announcement(&[], &cfg, GuardScope::Sync),
        None,
        "announcing an override on every run that did not need one is the same defect \
             pointing the other way"
    );
    let said = announcement(
        &[Objection::TooMany {
            count: 40,
            limit: 10,
            setting: "max_extra_removals",
        }],
        &cfg,
        GuardScope::PurgeUndeclared,
    )
    .expect("an objection a flag cleared is announced");
    assert!(said.contains("managed resources"), "{said}");
    assert!(
        said.contains("purge-undeclared"),
        "the line has to name the command that was allowed: {said}"
    );
}
