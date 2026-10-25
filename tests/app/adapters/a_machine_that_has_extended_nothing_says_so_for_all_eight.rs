use super::*;

#[test]
fn a_machine_that_has_extended_nothing_says_so_for_all_eight() {
    let (_d, layout) = layout_with(&[]);
    let all = survey(&layout);
    assert_eq!(all.len(), SURFACES.len());
    assert!(all.iter().all(|e| e.standing == Standing::Absent));
    assert!(all.iter().all(|e| !e.standing.is_wrong()));
}
