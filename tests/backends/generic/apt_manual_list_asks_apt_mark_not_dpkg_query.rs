use super::*;

#[tokio::test]
async fn apt_manual_list_asks_apt_mark_not_dpkg_query() {
    // The bug: apt had no manual command, so `list_manual` fell back to `dpkg-query
    // -W` — every installed package, dependencies included (579 vs 103 on the real
    // ubuntu image). It must ask `apt-mark showmanual` instead.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "apt-mark showmanual",
        Ok(DryRunOutput {
            stdout: b"apt\nbase-files\njq\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    // If it wrongly fell back, it would hit this instead — and adopt a dependency. That
    // sentence is the test's whole claim, and until this stub was registered as one that
    // must not be used, nothing checked it: the stub sat dead, and a product wired to the
    // wrong listing would have been caught only by the overlap between the two answers.
    mock.set_response_that_must_not_be_used(
        "dpkg-query -W -f=${db:Status-Status} ${Package} ${Version}\\n",
        Ok(DryRunOutput {
            stdout: b"installed apt 2.7.14\ninstalled jq 1.7.1\ninstalled libperl5.38t64 5.38.2\n"
                .to_vec(),
            stderr: vec![],
        }
        .into()),
    );

    let q = queryable_with(
        ManualListing::Command {
            binary: Some("apt-mark".into()),
            args: vec!["showmanual".into()],
            format: ManualFormat::BareNames,
        },
        mock.clone(),
        vfs,
    );

    let names: Vec<String> = q
        .list_manual()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(names, vec!["apt", "base-files", "jq"]);
    assert!(
        !names.contains(&"libperl5.38t64".to_string()),
        "a pure dependency must never be reported as user-chosen"
    );
    assert!(q.tracks_manual());

    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c == "apt-mark showmanual"),
        "{:?}",
        calls
    );
}
