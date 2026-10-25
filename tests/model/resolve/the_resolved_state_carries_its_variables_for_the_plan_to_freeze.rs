use super::*;

#[test]
fn the_resolved_state_carries_its_variables_for_the_plan_to_freeze() {
    // IX.6: a saved plan freezes these so `apply` reuses them instead of re-running a
    // provider that might read the clock and disagree with what the plan previewed.
    let f = fx(
        "Work
",
        &[(
            "Work",
            "apt:curl
",
        )],
        &[],
    );
    with_vars(
        &f,
        "role = travel
",
    );
    let vars = load_vars(&f).unwrap();
    let d = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts().with_vars(vars))
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .resolve()
        .unwrap();
    assert_eq!(
        d.vars["role"],
        shall::model::vars::Value::Str("travel".into())
    );
}
