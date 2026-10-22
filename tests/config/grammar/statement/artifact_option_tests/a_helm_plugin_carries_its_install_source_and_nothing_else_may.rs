use super::*;

#[test]
fn a_helm_plugin_carries_its_install_source_and_nothing_else_may() {
    // U39. The unit tests for this built a `PackageSpec` by hand and so never asked the
    // grammar whether `@url` was a legal key — it was not, and a real `helm` said so.
    assert_eq!(
        options_of("helm:diff@url=https://github.com/databus23/helm-diff").one("url"),
        Some("https://github.com/databus23/helm-diff")
    );
    let err = p("apt:curl@url=https://example.com/curl").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("not an option on `apt`"), "{}", msg);
    assert!(
        err.hint.unwrap().contains("helm"),
        "the refusal names who takes it"
    );
}
