use super::*;

#[test]
fn a_module_cannot_do_set_math() {
    // II.3: a module is a list. Choosing is the profile's job (V.2).
    for line in ["-vim", "exclude heavy", "intersect secure", "(a | b)"] {
        let f = fx(
            "Work\n",
            &[("Work", "use base\n")],
            &[("base.txt", &format!("apt:curl\n{}\n", line))],
        );
        let err = resolve(&f).unwrap_err();
        assert!(
            err.to_string().contains("module cannot"),
            "`{}` in a module must be refused, got: {}",
            line,
            err
        );
    }
}
