use super::*;

#[tokio::test]
async fn a_rendered_template_is_read_back_rendered_not_raw() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("gitconfig.tmpl");
    let dest = dir.path().join("gitconfig");
    std::fs::write(&source, "os={{ OS }}\n").unwrap();
    let rendered = format!("os={}\n", std::env::consts::OS);

    let config = Arc::new(shall::config::Config::default());
    let reg = BackendRegistry::new();
    let exec = CommandExecutor::new(false, false);
    let mut opts = Options::default();
    opts.insert("target".to_string(), dest.to_string_lossy().to_string());
    opts.insert("template".to_string(), "true".to_string());
    let stmt = Statement::Link(source.to_string_lossy().to_string(), opts);
    let key = ExtraKey::link(&dest);

    // Matching the render: in effect, no work.
    std::fs::write(&dest, &rendered).unwrap();
    assert_eq!(
        in_effect(&config, &reg, &exec, &stmt, &key).await,
        Some(true)
    );
    // Holding the raw template: drift, not in-effect. This is the comparison that
    // used to be impossible, which filed every template under `unverifiable`.
    std::fs::write(&dest, "os={{ OS }}\n").unwrap();
    assert_eq!(
        in_effect(&config, &reg, &exec, &stmt, &key).await,
        Some(false),
        "the destination holds the unrendered source, which is not the declaration"
    );
    // Hand-edited to something else: drift too.
    std::fs::write(&dest, "os=plan9\n").unwrap();
    assert_eq!(
        in_effect(&config, &reg, &exec, &stmt, &key).await,
        Some(false)
    );
    // A template that will not render cannot be compared: unverifiable, which places.
    std::fs::write(&source, "{% if %}unclosed\n").unwrap();
    assert_eq!(in_effect(&config, &reg, &exec, &stmt, &key).await, None);
    // A source that is not on disk is not in effect — the B0b rule, same as plain.
    std::fs::remove_file(&source).unwrap();
    assert_eq!(
        in_effect(&config, &reg, &exec, &stmt, &key).await,
        Some(false)
    );
}
