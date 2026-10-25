use super::*;

#[test]
fn a_condition_that_tests_no_variable_is_not_recorded() {
    // The tag answers "which variable put this here?". A `when host == laptop` has no
    // second hop to explain, and listing it would bury the ones that do.
    let f = fx(
        "Work\n",
        &[("Work", "use m\n")],
        &[("m.txt", "when host == laptop {\n  apt:curl\n}\n")],
    );
    let d = resolve(&f).unwrap();
    let curl = d.present().find(|p| p.name == "curl").unwrap();
    assert!(!curl.options.contains("__gated_by"));
}
