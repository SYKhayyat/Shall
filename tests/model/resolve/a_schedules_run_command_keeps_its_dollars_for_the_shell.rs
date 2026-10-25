use super::*;

#[test]
fn a_schedules_run_command_keeps_its_dollars_for_the_shell() {
    // `run` is a command line; `$` there belongs to the shell that will execute it.
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
    std::fs::write(
        f.layout.schedules_file(),
        "schedule:t@cron=0 2 * * *,run=sh -c 'echo $HOME'
",
    )
    .unwrap();
    let vars = load_vars(&f).unwrap();
    let d = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts().with_vars(vars))
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .resolve()
        .unwrap();
    let (_, opts, _) = d.schedules().next().unwrap();
    assert_eq!(opts.one("run"), Some("sh -c 'echo $HOME'"));
}
