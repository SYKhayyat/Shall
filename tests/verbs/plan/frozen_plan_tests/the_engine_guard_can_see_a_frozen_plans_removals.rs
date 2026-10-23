use super::*;

/// And the pair the engine's guard reads: `guard::removal_pairs` over this graph must find
/// the removals, because `apply` no longer calls the guard itself — it hands the plan to
/// `SyncEngine::sync`, whose first act is to enforce over exactly this.
#[test]
fn the_engine_guard_can_see_a_frozen_plans_removals() {
    let removals = vec![
        PlanRemoval {
            backend: "apt".into(),
            name: "vim".into(),
        },
        PlanRemoval {
            backend: "brew".into(),
            name: "fd".into(),
        },
    ];
    let changes = saved_plan_to_changes(&[spec("apt", "htop", &[])], &removals);
    let mut pairs = shall::app::sync::guard::removal_pairs(&changes);
    pairs.sort();
    assert_eq!(
        pairs,
        vec![
            ("apt".to_string(), "vim".to_string()),
            ("brew".to_string(), "fd".to_string())
        ],
        "the guard the engine runs must see every removal the plan froze"
    );
}
