use super::*;

/// **A template engine's behaviour is the whole reason it is a dependency, and nothing
/// here rendered anything.**
///
/// `@template=true` is the one mode that runs Tera, and its only test asserted that the
/// mode *reads its source* — never that the source comes out rendered. So `tera 1 -> 2`,
/// a major version of a template language, crossed every gate this repo has: it compiled,
/// so `build` and `clippy` were happy, and no test called the function.
///
/// Asserted against `std::env::consts` rather than a literal, because the point is that
/// the substitution happened at all — a build that rendered `{{ OS }}` to the empty string,
/// or left it as `{{ OS }}`, fails this either way, and the test does not need rewriting
/// on a different host.
#[tokio::test]
async fn a_template_is_rendered_with_the_facts_about_this_machine() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("gitconfig.tmpl");
    let target = dir.path().join("gitconfig");
    tokio::fs::write(&source, "os={{ OS }}\narch={{ ARCH }}\n")
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

    installer()
        .install(std::slice::from_ref(&spec), false)
        .await
        .unwrap();

    let written = tokio::fs::read_to_string(&target).await.unwrap();
    assert_eq!(
        written,
        format!(
            "os={}\narch={}\n",
            std::env::consts::OS,
            std::env::consts::ARCH
        ),
        "the template reached the target unrendered, or rendered to the wrong thing"
    );
}
