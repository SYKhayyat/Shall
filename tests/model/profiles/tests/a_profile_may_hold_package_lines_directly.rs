use super::*;

#[test]
fn a_profile_may_hold_package_lines_directly() {
    // II.4/V.3, accepted knowingly: `--into Work` is a real want, and the cost is that
    // a module can never reach these.
    let f = fixture(&[("Work", "use editors\napt:slack\n")], &[]);
    let r = resolve(&f, "Work").unwrap();
    assert_eq!(module_names(&r), ["editors"]);
    assert_eq!(r.direct.len(), 1);
}
