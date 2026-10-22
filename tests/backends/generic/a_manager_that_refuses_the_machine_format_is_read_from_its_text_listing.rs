use super::*;

/// Q43: **a manager too old for the machine-readable listing must fall back, not vanish.**
///
/// `--format json` needs dotnet SDK 10, `--json` a recent pixi, and Shall does not choose
/// which is installed. An unsupported flag exits non-zero with a usage message, and every
/// other reader here hands that back as an empty result — so asking for the better format
/// without negotiating would report an empty machine to exactly the users on older
/// tooling, which is `Q40` wearing a new hat.
#[tokio::test]
async fn a_manager_that_refuses_the_machine_format_is_read_from_its_text_listing() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    // The new flag is rejected, the way an older CLI rejects one it has never heard of.
    mock.set_response(
        "dpkg-query --format json",
        Ok(shall::core::executor::spoken_failure(
            2,
            "",
            "error: unexpected argument '--format' found",
        )),
    );
    // ...and the listing it does understand still answers.
    mock.set_response(
        "dpkg-query -W -f=${db:Status-Status} ${Package} ${Version}\n",
        Ok(DryRunOutput {
            stdout: b"installed jq 1.7.1
installed ripgrep 15.2.0
"
            .to_vec(),
            stderr: vec![],
        }
        .into()),
    );

    let mut core = apt_like_core(mock.clone(), vfs);
    core.config.list_binary = Some("dpkg-query".into());
    core.config.list_args = vec![
        "-W".into(),
        "-f=${db:Status-Status} ${Package} ${Version}\n".into(),
    ];
    core.config.machine_list = Some(MachineListing {
        binary: None,
        args: vec!["--format".into(), "json".into()],
        parse: std::sync::Arc::new(|_: &str| Ok(vec![Package::new("WRONG", "apt")])),
    });
    core.parser = Arc::new(shall::parsers::apt::AptParser);
    let q = GenericQueryable {
        core: Arc::new(core),
    };

    let names: Vec<String> = q
        .list_installed()
        .await
        .expect("a refused flag is not a failed listing")
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(
        names,
        vec!["jq", "ripgrep"],
        "the text listing must answer when the machine format is refused"
    );
    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c.contains("--format json")),
        "the machine format was never asked for: {calls:?}"
    );
}
