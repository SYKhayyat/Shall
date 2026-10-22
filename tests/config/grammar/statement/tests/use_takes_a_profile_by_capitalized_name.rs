use super::*;

#[test]
fn use_takes_a_profile_by_capitalized_name() {
    assert_eq!(
        p("use Work").unwrap(),
        Statement::Use(Reference::Profile("Work".into()), vec![])
    );
}
