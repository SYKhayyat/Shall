use super::*;

#[tokio::test]
async fn an_os_essential_package_is_a_live_line() {
    // II.9, ruled 2026-08-05. The commented-out section guarded against a deletion the
    // guard already refuses (V.26) — and bought that redundancy by leaving the machine's
    // load-bearing packages undeclared, so drift in them was invisible.
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    mock.set_response(
        "apt-mark showmanual",
        Ok(DryRunOutput {
            stdout: b"jq\nbash\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    // `Q47` ruled that adoption does not consult the guard, so this stub going unused *is*
    // the proof of that ruling — and nothing checked it until it was registered as one that
    // must not be used. Its content is dpkg's real format: bash is Essential, jq is not.
    mock.set_response_that_must_not_be_used(
        "dpkg-query -W -f=${Essential} ${Priority} ${Package}\\n",
        Ok(DryRunOutput {
            stdout: b"yes required bash\nno optional jq\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let reg = registry_with(
        ManualListing::Command {
            binary: Some("apt-mark".into()),
            args: vec!["showmanual".into()],
            format: ManualFormat::BareNames,
        },
        mock,
    );
    let m = Adopter::new(
        over(reg),
        Arc::new(Mutex::new(StateRegistry::default())),
        &Config::default(),
    );
    let text = m.render_manifest(&m.discover().await.unwrap(), &m.adoption_options());

    assert!(text.contains("\napt:jq\n"), "jq is a live line:\n{}", text);
    assert!(
        text.contains("\napt:bash\n"),
        "an OS-essential package is a live line too:\n{}",
        text
    );
    assert!(
        !text.contains("#   apt:bash — "),
        "and it is not also commented out:\n{}",
        text
    );
    // The header must not promise a deletion the guard will refuse.
    assert!(
        text.contains("Except where the guard refuses"),
        "the header owes the reader the exception:\n{}",
        text
    );
}
