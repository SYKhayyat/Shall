use super::*;

/// Every shape NixOS cannot express, refused by name rather than approximated (P7). Each of
/// these produced a *different* wrong answer before: a policy silently ignored, a service
/// enabled when it was declared stopped, a perimeter half applied.
#[test]
fn a_line_this_os_cannot_express_is_refused_by_name() {
    let cases: Vec<(Statement, &str)> = vec![
        (
            Statement::Firewall("default/outgoing".into(), Options::default()),
            "outgoing",
        ),
        (
            Statement::Firewall("default/incoming".into(), opts(&[("value", "maybe")])),
            "policy",
        ),
        (
            Statement::Service(
                "nginx".into(),
                opts(&[("enabled", "false"), ("status", "running")]),
            ),
            "enable",
        ),
    ];
    for (stmt, needle) in cases {
        let err = declared(&state(vec![stmt.clone()]))
            .expect_err(&format!("{:?} must be refused", stmt))
            .to_string();
        assert!(err.contains(needle), "{err}");
        // And the refusal has to say which line, or a user with forty modules cannot act
        // on it.
        assert!(err.contains("modules/test.txt"), "{err}");
    }
}
