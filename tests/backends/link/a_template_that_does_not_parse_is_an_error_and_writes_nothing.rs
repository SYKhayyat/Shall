use super::*;

/// The other half of the same promise: a template that does not parse must be reported as
/// a template that does not parse, rather than written through verbatim.
#[tokio::test]
async fn a_template_that_does_not_parse_is_an_error_and_writes_nothing() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("broken.tmpl");
    let target = dir.path().join("broken");
    tokio::fs::write(&source, "{% if %}unclosed\n")
        .await
        .unwrap();

    let mut options = shall::config::grammar::Options::default();
    options.set("target", target.to_string_lossy().to_string());
    options.set("template", "true".to_string());
    let spec = PackageSpec {
        name: source.to_string_lossy().to_string(),
        backend: "link".into(),
        options,
        requires: vec![],
        present: true,
    };

    let err = installer()
        .install(std::slice::from_ref(&spec), false)
        .await
        .expect_err("a template that does not parse must not be reported as installed");
    assert!(
        err.to_string().contains("Tera"),
        "the error does not say the template was the problem: {err}"
    );
    assert!(
        !tokio::fs::try_exists(&target).await.unwrap(),
        "a template that did not render still wrote its target"
    );
}
