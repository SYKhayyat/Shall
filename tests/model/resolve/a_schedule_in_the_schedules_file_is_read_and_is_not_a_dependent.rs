use super::*;

#[test]
fn a_schedule_in_the_schedules_file_is_read_and_is_not_a_dependent() {
    // S21 parts (1) and (3): the resolver reads the `schedules` file, and its lines are
    // schedules — not dependents, not packages.
    let f = fx("Work\n", &[("Work", "apt:nginx\n")], &[]);
    std::fs::write(
        f.layout.schedules_file(),
        "schedule:nightly@cron=0 2 * * *,run=clean\n",
    )
    .unwrap();
    let d = resolve(&f).unwrap();
    assert_eq!(d.schedules().count(), 1);
    let (name, _opts, _origin) = d.schedules().next().unwrap();
    assert_eq!(name, "nightly");
    // A schedule is never a dependent (II.7 phase 4, not phase 3).
    assert!(!d.has_dependents());
}
