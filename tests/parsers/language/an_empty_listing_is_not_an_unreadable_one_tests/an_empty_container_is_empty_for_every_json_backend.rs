use super::*;

/// And an empty container of every shape is still an empty machine.
#[test]
fn an_empty_container_is_empty_for_every_json_backend() {
    for (backend, empty) in [
        ("npm", r#"{"dependencies":{}}"#),
        ("pnpm", r#"[{"dependencies":{}}]"#),
        ("pip", "[]"),
        ("pipx", r#"{"pipx_spec_version":"0.1","venvs":{}}"#),
        ("composer", r#"{"installed":[]}"#),
    ] {
        assert_eq!(
            parse_installed(backend, empty)
                .unwrap_or_else(|e| panic!("{backend} read an empty machine as unread: {e}")),
            Vec::new(),
            "{backend}"
        );
    }
}
