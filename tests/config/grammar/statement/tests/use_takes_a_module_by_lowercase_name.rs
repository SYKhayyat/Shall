use super::*;

#[test]
fn use_takes_a_module_by_lowercase_name() {
    assert_eq!(
        p("use editors").unwrap(),
        Statement::Use(Reference::Module("editors".into()), vec![])
    );
}
