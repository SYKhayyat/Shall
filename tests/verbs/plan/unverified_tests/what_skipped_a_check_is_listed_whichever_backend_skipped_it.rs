use super::*;

/// Every backend the flag is legal on stays visible after the install — the download ones
/// and, since Q5, the manager that verifies a signature itself.
#[test]
fn what_skipped_a_check_is_listed_whichever_backend_skipped_it() {
    let mut state = shall::core::StateRegistry::default();
    state.set_managed([
        pkg("helm", "diff", true),
        pkg("github", "sharkdp/fd", true),
        pkg("web", "https://example.com/tool", true),
        pkg("appimage", "https://example.com/x.AppImage", true),
        pkg("apt", "curl", false),
        pkg("github", "BurntSushi/ripgrep", false),
    ]);

    let listed = unverified_packages(&state);
    assert_eq!(
        listed,
        vec![
            // `(backend, name)` order: the registry is keyed by that pair, so the listing
            // is stable across runs rather than following whatever order the rows were
            // recorded in.
            (
                "appimage".to_string(),
                "https://example.com/x.AppImage".to_string()
            ),
            ("github".to_string(), "sharkdp/fd".to_string()),
            ("helm".to_string(), "diff".to_string()),
            ("web".to_string(), "https://example.com/tool".to_string()),
        ],
        "the listing must name exactly what skipped a check"
    );
}
