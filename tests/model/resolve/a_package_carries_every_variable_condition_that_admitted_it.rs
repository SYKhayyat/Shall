use super::*;

#[test]
fn a_package_carries_every_variable_condition_that_admitted_it() {
    // W11's gating half. The chain crosses three files — `active` turned the profile on,
    // the profile's block let the `use` through, the module's block let the line
    // through — and `why` needs all three, in that order.
    let f = fx(
        "when $role == travel {\n  Trip\n}\n",
        &[("Trip", "when $tier == full {\n  use m\n}\n")],
        &[("m.txt", "when $gpu == true {\n  apt:mosh\n}\napt:curl\n")],
    );
    with_vars(&f, "role = travel\ntier = full\ngpu = true\n");
    let vars = load_vars(&f).unwrap();
    let d = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts().with_vars(vars))
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .resolve()
        .unwrap();

    let mosh = d.present().find(|p| p.name == "mosh").unwrap();
    let chain: Vec<&str> = mosh
        .options
        .all("__gated_by")
        .iter()
        .map(String::as_str)
        .collect();
    assert_eq!(chain.len(), 3, "{:?}", chain);
    assert!(
        chain[0].starts_with("when $role == travel @ "),
        "{:?}",
        chain
    );
    assert!(chain[1].starts_with("when $tier == full @ "), "{:?}", chain);
    assert!(chain[2].starts_with("when $gpu == true @ "), "{:?}", chain);

    // `curl` is outside the module's block but still inside the two that led here.
    let curl = d.present().find(|p| p.name == "curl").unwrap();
    assert_eq!(curl.options.all("__gated_by").len(), 2);
}
