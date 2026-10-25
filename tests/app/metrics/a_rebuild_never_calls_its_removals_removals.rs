use super::*;

#[test]
fn a_rebuild_never_calls_its_removals_removals() {
    // K15. All of them come straight back, and a bare "Removals: 214" is the sentence
    // that reads as a machine being dismantled.
    let (_, removals) = summary_labels(Narration::Rebuild);
    assert!(!removals.eq_ignore_ascii_case("removals"));
    assert!(removals.contains("reinstall"));
    assert_eq!(summary_labels(Narration::Change).1, "Removals");
}
