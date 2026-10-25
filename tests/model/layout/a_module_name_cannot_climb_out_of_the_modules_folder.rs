use super::*;

#[test]
fn a_module_name_cannot_climb_out_of_the_modules_folder() {
    // SEC6. `module add --name ../../foo` wrote outside `modules/`; the type is what
    // stops it, so no call site can lose the check.
    let err = ModuleName::new("../../foo").unwrap_err();
    assert!(err.contains("is not a module name"), "{}", err);
    assert!(
        err.contains("lowercase letters, digits"),
        "teach the rule: {}",
        err
    );

    for bad in ["", "a/b", "a\\b", "a.b", "..", "a b", "Ünicode"] {
        assert!(ModuleName::new(bad).is_err(), "`{}` must be refused", bad);
    }
    for good in ["editors", "Editors", "web-dev", "py_3", "x9"] {
        assert!(ModuleName::new(good).is_ok(), "`{}` must be accepted", good);
    }
}
