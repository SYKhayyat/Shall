use super::*;

/// The fixture is that command's real output, captured from the openSUSE image.
#[tokio::test]
async fn preamble_and_metadata_are_not_dependencies() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/zypper/info-requires.txt"
    ))
    .expect("the zypper fixture");

    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "zypper info --requires -- jq",
        Ok(DryRunOutput {
            stdout: fixture.into_bytes(),
            stderr: vec![],
        }
        .into()),
    );
    let exec = CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new()));
    let mut core = apt_like_core_named("zypper", exec);
    core.config.depends = Some(DependsProbe {
        binary: None,
        args: vec!["info".into(), "--requires".into(), "{name}".into()],
        parse: Arc::new(parse_dependency_output),
    });

    let deps = core.get_dependencies("jq").await.unwrap();

    for junk in [
        "Loading",
        "Reading",
        "No",
        "Information",
        "openSUSE",
        "openSUSE-Tumbleweed-Oss",
        "jq-1.8.2-1.3.src",
        "[4]",
        "A",
        "JSON",
        "not",
    ] {
        assert!(
            !deps.iter().any(|d| d == junk),
            "`{junk}` is a word zypper printed, not a package it requires — got {deps:?}"
        );
    }
    assert_eq!(
        deps,
        vec![
            "libc.so.6(GLIBC_2.38)(64bit)".to_string(),
            "libonig.so.5()(64bit)".to_string(),
            "libjq.so.1()(64bit)".to_string(),
            "libjq1".to_string(),
        ],
        "only the four lines under `Requires` are requirements"
    );
}
