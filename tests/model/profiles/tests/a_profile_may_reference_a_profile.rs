use super::*;

#[test]
fn a_profile_may_reference_a_profile() {
    // II.7 step 2. The opposite direction is the one that is forbidden.
    let f = fixture(
        &[("Work", "use Base\nuse dev\n"), ("Base", "use editors\n")],
        &[],
    );
    assert_eq!(
        module_names(&resolve(&f, "Work").unwrap()),
        ["editors", "dev"]
    );
}
