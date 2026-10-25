use super::*;

#[test]
fn usable_is_the_priority_order_and_nothing_else() {
    let reg = registry_of(&["apt", "cargo", "npm"]);
    let b = Backends::new(reg, Ok(Priority::from_backends(vec!["cargo".into()])));
    let names: Vec<String> = b
        .usable()
        .unwrap()
        .iter()
        .map(|x| x.name().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["cargo"],
        "a backend `priority` does not name must not be used, however installed it is"
    );
}
