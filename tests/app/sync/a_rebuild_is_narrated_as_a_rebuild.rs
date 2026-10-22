/// **A rebuild is narrated as a rebuild, and a change as a change.**
///
/// Found by the mutation gate: the `GuardScope::Rebuild` arm of `narration_for` deleted
/// survived the whole suite. **The narration is what the summary counts**, and a rebuild's two
/// transactions run through this engine like any other sync — so the summary has to be told
/// which run it is narrating, or it reports a rebuild's removals as removals.
///
/// The two labels are not decoration: `summary_labels` turns them into "Installs"/"Removals"
/// and "Reinstalled"/"Removed to reinstall", and a rebuild that says "Removals" reads as a
/// removal of things the user never asked to have removed.
#[test]
fn a_rebuild_is_narrated_as_a_rebuild() {
    assert_eq!(
        shall::app::sync::narration_for(shall::app::sync::guard::GuardScope::Rebuild),
        shall::app::metrics::Narration::Rebuild,
        "a rebuild narrated as a change reports its removals as removals"
    );
    assert_eq!(
        shall::app::sync::narration_for(shall::app::sync::guard::GuardScope::Sync),
        shall::app::metrics::Narration::Change,
        "an ordinary sync is a change, and every other scope is too"
    );
    assert_eq!(
        shall::app::sync::narration_for(shall::app::sync::guard::GuardScope::Apply),
        shall::app::metrics::Narration::Change,
    );
}
