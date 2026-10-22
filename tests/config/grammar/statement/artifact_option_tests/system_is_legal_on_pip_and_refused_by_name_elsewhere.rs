use super::*;

/// `Q49`. `@system` is legal exactly where a manager can be told to write into an
/// environment the OS owns, and refused by name everywhere else — an option accepted on a
/// backend that has no such notion is an option that does nothing and says nothing.
#[test]
fn system_is_legal_on_pip_and_refused_by_name_elsewhere() {
    assert_eq!(
        options_of("pip:black@system=true").one("system"),
        Some("true")
    );

    for line in ["apt:jq@system=true", "cargo:ripgrep@system=true"] {
        let err = p(line).expect_err("`@system` means nothing here and must be refused");
        let text = format!("{err}");
        assert!(text.contains("@system"), "{text}");
        assert!(
            text.contains("pip"),
            "the refusal has to name where it IS legal: {text}"
        );
    }
}
