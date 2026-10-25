use super::*;

/// #69's motivating case: a template names the home directory it is rendered for. The
/// context answers `HOME` from the detected fact — the same directory `~/` targets
/// expand through — so `topdirs = {{ HOME }}/Documents` is per-user by construction.
#[tokio::test]
async fn a_template_names_the_home_it_is_rendered_for() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("recoll.tmpl");
    let target = dir.path().join("recoll.conf");
    tokio::fs::write(&source, "topdirs = {{ HOME }}/Documents\n")
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

    let home = dirs::home_dir().expect("this test needs a detectable home");
    assert_eq!(
        tokio::fs::read_to_string(&target).await.unwrap(),
        format!("topdirs = {}/Documents\n", home.display())
    );
}
