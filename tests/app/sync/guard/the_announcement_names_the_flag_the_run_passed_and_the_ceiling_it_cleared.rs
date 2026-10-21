use super::*;

/// The line names the flag **this run passed**, and the ceiling **that objection carried**.
///
/// Both halves used to come from somewhere else: the flag was the literal
/// `--allow-mass-removal` whatever was passed, and the noun was the caller's, so a run of
/// `sync --allow-mass-install` â€” which answers `max_total_changes` and nothing else (`N8`) â€”
/// read *"the removal count â€¦ was allowed by --allow-mass-removal"*, naming a ceiling it had
/// not cleared and a flag it had not been given (`J9`).
///
/// **Asserted with `!contains` in both directions.** A sentence that names both flags every
/// time passes every `contains` here and is exactly as wrong as the literal it replaced;
/// only the absent half distinguishes them.
#[test]
fn the_announcement_names_the_flag_the_run_passed_and_the_ceiling_it_cleared() {
    let total = |c, l| Objection::TooMany {
        count: c,
        limit: l,
        setting: TOTAL_KEY,
    };
    let per_kind = |c, l| Objection::TooMany {
        count: c,
        limit: l,
        setting: "max_removals",
    };

    // Install-only: the flag answers the total, so the total is what the line may name.
    let mut cfg = config_with(10);
    cfg.allow_mass_install = true;
    let said = announcement(&[total(62, 50)], &cfg, GuardScope::Sync).expect("a cleared count");
    assert!(said.contains("--allow-mass-install"), "{said}");
    assert!(
        !said.contains("--allow-mass-removal"),
        "a run that never typed it must not be told it was used: {said}"
    );
    assert!(
        said.contains("makes 62 changes in total") && said.contains(TOTAL_KEY),
        "the ceiling comes off the objection, not off the caller's noun: {said}"
    );
    assert!(
        !said.contains("removal count"),
        "an install-only run removed nothing: {said}"
    );

    // Removal-only, the mirror image, on a per-kind ceiling only that flag can clear.
    let mut cfg = config_with(10);
    cfg.allow_mass_removal = true;
    let said = announcement(&[per_kind(40, 10)], &cfg, GuardScope::Sync).expect("a cleared count");
    assert!(said.contains("--allow-mass-removal"), "{said}");
    assert!(
        !said.contains("--allow-mass-install"),
        "the sentence must not name every flag that exists: {said}"
    );
    assert!(said.contains("removes 40 packages"), "{said}");

    // Both passed: both named, because for the one ceiling either answers, either did.
    let mut cfg = config_with(10);
    cfg.allow_mass_removal = true;
    cfg.allow_mass_install = true;
    let said = announcement(&[total(62, 50)], &cfg, GuardScope::Sync).expect("a cleared count");
    assert!(
        said.contains("--allow-mass-removal") && said.contains("--allow-mass-install"),
        "{said}"
    );

    // Two ceilings cleared at once: both are named, as the refusal names both when they
    // stand. A run told about one of them raises that number and meets the other.
    let said = announcement(&[per_kind(40, 10), total(62, 50)], &cfg, GuardScope::Sync)
        .expect("a cleared count");
    assert!(
        said.contains("max_removals") && said.contains(TOTAL_KEY),
        "{said}"
    );

    // Neither flag: there is no sentence at all, rather than a sentence with a placeholder
    // where the flag belongs. An announcement is a report that an override was used, so a
    // run that used none has nothing to announce however many objections it is handed.
    let cfg = config_with(10);
    assert_eq!(
        announcement(&[total(62, 50)], &cfg, GuardScope::Sync),
        None,
        "no flag was passed, so nothing allowed anything"
    );
}
