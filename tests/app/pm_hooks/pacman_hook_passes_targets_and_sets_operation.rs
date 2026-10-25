use super::*;

#[test]
fn pacman_hook_passes_targets_and_sets_operation() {
    let specs = hook_specs("shall");
    let install = specs
        .iter()
        .find(|s| s.manager == "pacman" && s.content.contains("--op install"))
        .unwrap();
    assert!(install.content.contains("NeedsTargets"));
    assert!(install.content.contains("Operation = Install"));
    assert!(install.content.contains("Operation = Upgrade"));
    assert!(install.content.contains("xargs"));
}
