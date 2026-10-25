use super::*;

/// The order is a fact the resolver depends on: a bare name goes to the first manager in
/// the file that has it.
#[test]
fn usable_keeps_the_file_order_rather_than_the_registrys() {
    // The registry is a BTreeMap, so its own order is alphabetical — the reverse of this.
    let reg = registry_of(&["apt", "cargo", "npm"]);
    let b = Backends::new(
        reg,
        Ok(Priority::from_backends(vec![
            "npm".into(),
            "cargo".into(),
            "apt".into(),
        ])),
    );
    let names: Vec<String> = b
        .usable()
        .unwrap()
        .iter()
        .map(|x| x.name().to_string())
        .collect();
    assert_eq!(names, vec!["npm", "cargo", "apt"]);
}
