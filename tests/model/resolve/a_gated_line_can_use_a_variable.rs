use super::*;

#[test]
fn a_gated_line_can_use_a_variable() {
    let f = fx(
        "Work
",
        &[(
            "Work", "use m
",
        )],
        &[(
            "m.txt",
            "apt:curl
when $role == travel {
  apt:mosh
}
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
    assert_eq!(names(&d, "apt"), vec!["curl", "mosh"]);
}
