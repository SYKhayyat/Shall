use super::*;

#[test]
fn every_surface_is_named_once_and_resolves_by_name() {
    let mut names: Vec<&str> = SURFACES.iter().map(|s| s.name).collect();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count, "two surfaces share a name");
    for s in &SURFACES {
        assert!(surface(s.name).is_some());
    }
    assert!(surface("nonesuch").is_none());
}
