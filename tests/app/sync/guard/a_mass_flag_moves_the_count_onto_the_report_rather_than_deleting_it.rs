use super::*;

/// What a mass flag answered is **on the report**, not only in the log.
///
/// The three states this distinguishes look identical from the objection list alone: an
/// objection that was never raised, one that was raised and cleared, and one that was raised
/// and stands. Two of the three end with the same empty list, which is why the only witness
/// used to be a `warn!` â€” and a line nothing can read is a fact nothing can check.
#[tokio::test]
async fn a_mass_flag_moves_the_count_onto_the_report_rather_than_deleting_it() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(2);
    let over_the_limit = pairs(&["a", "b", "c"]);

    // No flag: the count objects, and nothing has been answered.
    let mut report = inspect_removals(
        &cfg,
        &reg,
        &over_the_limit,
        RemovalKind::Package,
        &Reaping::new(),
        &mut EssentialsCache::default(),
    )
    .await;
    allow_the_count(&cfg, &mut report, GuardScope::Sync);
    assert!(
        matches!(
            report.objections.as_slice(),
            [Objection::TooMany { setting, .. }] if *setting == "max_removals"
        ),
        "{:?}",
        report.objections
    );
    assert!(
        report.allowed_by_flag.is_empty(),
        "no flag was passed, so nothing was allowed: {:?}",
        report.allowed_by_flag
    );

    // With it: the same objection is answered, and the report says which one and how far
    // over it was â€” the numbers, so a permutation of the list cannot pass for a clearance.
    cfg.allow_mass_removal = true;
    let mut report = inspect_removals(
        &cfg,
        &reg,
        &over_the_limit,
        RemovalKind::Package,
        &Reaping::new(),
        &mut EssentialsCache::default(),
    )
    .await;
    allow_the_count(&cfg, &mut report, GuardScope::Sync);
    assert!(report.is_empty(), "{:?}", report.objections);
    assert!(
        matches!(
            report.allowed_by_flag.as_slice(),
            [Objection::TooMany {
                count: 3,
                limit: 2,
                setting,
            }] if *setting == "max_removals"
        ),
        "{:?}",
        report.allowed_by_flag
    );

    // A protection is not a count. The flag neither clears it nor claims to have.
    let mut report = inspect_removals(
        &cfg,
        &reg,
        &pairs(&["python3"]),
        RemovalKind::Package,
        &Reaping::new(),
        &mut EssentialsCache::default(),
    )
    .await;
    allow_the_count(&cfg, &mut report, GuardScope::Sync);
    assert!(
        matches!(report.objections.as_slice(), [Objection::Protected { .. }]),
        "{:?}",
        report.objections
    );
    assert!(
        report.allowed_by_flag.is_empty(),
        "a refusal is not something a flag answers (V.26): {:?}",
        report.allowed_by_flag
    );
}
