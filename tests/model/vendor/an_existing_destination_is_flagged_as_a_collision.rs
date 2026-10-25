use super::*;

/// U14: a destination that already exists is a collision — named, not silently overwritten.
#[test]
fn an_existing_destination_is_flagged_as_a_collision() {
    let files = vec![PathBuf::from("modules/tools.txt")];
    let mine = |p: &Path| p == Path::new("modules/tools.txt");
    let plan = plan(&files, &mine);
    assert_eq!(plan.collisions, vec![PathBuf::from("modules/tools.txt")]);
    // The placement is still listed — `--force` acts on exactly these.
    assert_eq!(plan.placements.len(), 1);
}
