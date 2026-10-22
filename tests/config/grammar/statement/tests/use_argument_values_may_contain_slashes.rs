use super::*;

#[test]
fn use_argument_values_may_contain_slashes() {
    // The `/` is in an argument value, not the `use` target, so it is not a path.
    assert_eq!(
        p("use m(path=/etc/foo)").unwrap(),
        Statement::Use(
            Reference::Module("m".into()),
            vec![("path".into(), "/etc/foo".into())]
        )
    );
}
