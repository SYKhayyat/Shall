use super::*;

/// The other side: when the manager *does* support it, the text listing is never run.
/// Asking both every time would double every listing on every current machine.
#[tokio::test]
async fn a_manager_that_answers_the_machine_format_is_not_asked_twice() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "dpkg-query --format json",
        Ok(DryRunOutput {
            stdout: b"[]".to_vec(),
            stderr: vec![],
        }
        .into()),
    );

    let mut core = apt_like_core(mock.clone(), vfs);
    core.config.list_binary = Some("dpkg-query".into());
    core.config.list_args = vec!["-W".into()];
    core.config.machine_list = Some(MachineListing {
        binary: None,
        args: vec!["--format".into(), "json".into()],
        parse: std::sync::Arc::new(|_: &str| Ok(vec![Package::new("from-json", "apt")])),
    });
    core.parser = Arc::new(shall::parsers::apt::AptParser);
    let q = GenericQueryable {
        core: Arc::new(core),
    };

    let pkgs = q.list_installed().await.unwrap();
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "from-json");
    let calls = mock.get_calls().await;
    assert!(
        !calls.iter().any(|c| c == "dpkg-query -W"),
        "the text listing ran even though the machine format worked: {calls:?}"
    );
}
