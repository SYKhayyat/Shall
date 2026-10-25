use super::*;

/// The shipped rows go through the loader, not around it — K17's rule, which exists so the
/// built-ins exercise the same path a user's row does.
#[test]
fn the_shipped_table_parses_and_carries_three_firewalls() {
    let all = adapters(vec![]);
    let names: Vec<&str> = all.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"ufw"), "{:?}", names);
    assert!(names.contains(&"firewalld"), "{:?}", names);
    assert!(names.contains(&"windows-defender"), "{:?}", names);
}
