use super::*;

#[test]
fn dependents_are_the_after_package_extras_only() {
    // II.7 phase 3: shims, services and links are the dependent phase. `repo:` is phase
    // 1, so it is excluded. (`schedule:` can no longer appear in a module at all — see
    // `a_schedule_outside_the_schedules_file_is_refused` — so it is not part of this
    // fixture.)
    let f = fx(
        "Work\n",
        &[("Work", "use svc\n")],
        &[(
            "svc.txt",
            "repo:apt:ppa:deadsnakes/ppa\n\
             apt:nginx\n\
             service:nginx@enabled=true\n\
             link:~/.config/nginx.conf@target=~/.config/nginx.conf\n\
             shim:rg\n",
        )],
    );
    let d = resolve(&f).unwrap();
    assert!(d.has_dependents());
    let kinds: Vec<&str> = d
        .dependents()
        .map(|(s, _)| match s {
            Statement::Service(..) => "service",
            Statement::Link(..) => "link",
            Statement::Shim(..) => "shim",
            _ => "other",
        })
        .collect();
    // Declaration order preserved; repo excluded.
    assert_eq!(kinds, ["service", "link", "shim"]);
}
