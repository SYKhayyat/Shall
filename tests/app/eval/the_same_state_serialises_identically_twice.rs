use super::*;

/// The output is for diffing, so the same configuration must serialise identically twice —
/// otherwise every comparison is noise.
#[test]
fn the_same_state_serialises_identically_twice() {
    let mut state = shall::model::DesiredState::default();
    state.packages.insert(
        "apt".into(),
        vec![spec("apt", "a", true), spec("apt", "b", true)],
    );
    state
        .packages
        .insert("cargo".to_string(), vec![spec("cargo", "c", true)]);
    let a = serde_json::to_string(&Evaluation::of(&state, Path::new("/repo"))).unwrap();
    let b = serde_json::to_string(&Evaluation::of(&state, Path::new("/repo"))).unwrap();
    assert_eq!(a, b);
}
