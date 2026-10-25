use super::*;

#[test]
fn a_variable_expands_inside_an_option_value() {
    let f = fx(
        "Work
",
        &[(
            "Work", "use m
",
        )],
        &[(
            "m.txt",
            "apt:nginx@version=$pinned
",
        )],
    );
    with_vars(
        &f,
        "pinned = 1.24.0
",
    );
    let vars = load_vars(&f).unwrap();
    let d = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts().with_vars(vars))
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .resolve()
        .unwrap();
    let spec = d.present().find(|p| p.name == "nginx").unwrap();
    assert_eq!(spec.options.one("version"), Some("1.24.0"));
}
