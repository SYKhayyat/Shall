use super::*;

#[test]
fn a_variable_expands_inside_a_declaration_value() {
    let f = fx(
        "Work
",
        &[(
            "Work", "use m
",
        )],
        &[(
            "m.txt",
            "link:~/.config/${role}/init.lua
",
        )],
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
    let names: Vec<String> = d
        .dependents()
        .filter_map(|(s, _)| match s {
            Statement::Link(n, _) => Some(n.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["~/.config/travel/init.lua"]);
}
