use super::*;

#[test]
fn the_plan_places_shareable_files_and_skips_the_rest() {
    let files = vec![
        PathBuf::from("modules/tools.txt"),
        PathBuf::from("adapters/backends.toml"),
        PathBuf::from("profiles/Work"),
        PathBuf::from("README.md"),
    ];
    let plan = plan(&files, &|_| false);
    assert_eq!(plan.placements.len(), 2, "{:?}", plan.placements);
    assert!(plan.collisions.is_empty());
    assert_eq!(
        plan.placements[0].to,
        PathBuf::from("adapters/backends.toml")
    );
    assert_eq!(plan.placements[1].to, PathBuf::from("modules/tools.txt"));
}
