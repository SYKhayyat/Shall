use super::*;

#[test]
fn the_order_asked_is_the_order_written_then_priority() {
    let priority: Vec<String> = ["apt", "snap", "cargo"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(Candidates::Priority.order(&priority), priority);
    assert_eq!(
        Candidates::Named(vec!["cargo".into()]).order(&priority),
        vec!["cargo".to_string()],
        "a closed chain never reaches priority"
    );
    // The named head keeps its place and is not repeated when the tail names it again.
    assert_eq!(
        Candidates::NamedThenPriority(vec!["cargo".into()]).order(&priority),
        vec!["cargo".to_string(), "apt".to_string(), "snap".to_string()]
    );
}
