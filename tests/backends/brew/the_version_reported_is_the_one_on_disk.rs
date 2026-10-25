use super::*;

/// And the version is the keg's, not `versions.stable`. Reading the latest published
/// version made every `brew:x@version=` pin re-install on every sync from the moment
/// Homebrew published a newer bottle.
#[tokio::test]
async fn the_version_reported_is_the_one_on_disk() {
    let (mock, core) = mocked();
    mock.set_response(
        "brew info --json=v1 -- jq",
        Ok(shall::core::executor::DryRunOutput {
            stdout: brew_info_json(
                r#"[{"version":"1.6","prefix":"/opt/homebrew/Cellar/jq/1.6","installed_as_dependency":false}]"#,
            )
            .into_bytes(),
            stderr: vec![],
        }
        .into()),
    );
    let found = BrewQueryable { core }
        .info("jq")
        .await
        .unwrap()
        .expect("installed");
    assert_eq!(
        found.version.as_deref(),
        Some("1.6"),
        "the stable version is what an upgrade would move to, not what is installed"
    );
    assert_eq!(
        found.properties.get("install_path").map(String::as_str),
        Some("/opt/homebrew/Cellar/jq/1.6"),
        "the keg prefix still reaches the caller"
    );
    assert_eq!(
        found
            .properties
            .get("installed_as_dependency")
            .map(String::as_str),
        Some("false")
    );
}
