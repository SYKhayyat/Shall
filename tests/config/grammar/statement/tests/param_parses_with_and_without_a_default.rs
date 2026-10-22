use super::*;

#[test]
fn param_parses_with_and_without_a_default() {
    assert_eq!(
        p("param user").unwrap(),
        Statement::Param {
            name: "user".into(),
            default: None
        }
    );
    assert_eq!(
        p("param gpu = none").unwrap(),
        Statement::Param {
            name: "gpu".into(),
            default: Some("none".into())
        }
    );
}
