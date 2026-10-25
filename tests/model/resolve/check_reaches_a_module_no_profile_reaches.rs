use super::*;

#[test]
fn check_reaches_a_module_no_profile_reaches() {
    // II.3. A broken file nobody activated is still broken, and the day it is activated is
    // the worst day to find out. Both errors are returned, not just the first.
    let f = fx(
        "Work\n",
        &[("Work", "use good\n")],
        &[
            ("good.txt", "apt:curl\n"),
            ("orphan.txt", "apt:nginx@requires=libfoo\n"),
            ("other.txt", "-vim\n"),
        ],
    );
    // Resolution is clean: nothing reaches the broken files.
    assert_eq!(names(&resolve(&f).unwrap(), "apt"), ["curl"]);

    let errors = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(facts())
        .parse_everything();
    let found: Vec<String> = errors.iter().map(|e| e.origin.to_string()).collect();
    assert_eq!(errors.len(), 2, "{:?}", found);
    assert!(
        found.iter().any(|o| o.contains("orphan.txt")),
        "{:?}",
        found
    );
    assert!(found.iter().any(|o| o.contains("other.txt")), "{:?}", found);
}
