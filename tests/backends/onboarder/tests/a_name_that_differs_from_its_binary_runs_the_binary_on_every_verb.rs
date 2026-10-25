use super::*;

/// XIII.12: the prefix a line is written with and the program that runs are two facts.
/// `firewall:22/tcp` runs `ufw`, and every verb has to agree about that — an install that
/// ran `ufw` while the removal ran `firewall` would leave a rule nothing can take back.
#[tokio::test]
async fn a_name_that_differs_from_its_binary_runs_the_binary_on_every_verb() {
    let (mock, exec) = mock_exec();
    let mut reg = BackendRegistry::new();
    assert_eq!(
        register_custom_backends(&mut reg, &exec, vec![firewall_def()]),
        1
    );
    let caps = reg.get("firewall").expect("firewall registered");

    caps.as_installable()
        .unwrap()
        .install(
            &[shall::core::PackageSpec {
                name: "22/tcp".into(),
                backend: "firewall".into(),
                options: Default::default(),
                requires: vec![],
                present: true,
            }],
            false,
        )
        .await
        .unwrap();
    caps.as_queryable().unwrap().list_installed().await.unwrap();
    caps.as_installable()
        .unwrap()
        .remove(
            &["22/tcp".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    assert_eq!(calls.len(), 3, "{:?}", calls);
    assert!(calls.iter().all(|c| c.starts_with("ufw ")), "{:?}", calls);
    assert!(
        calls.iter().any(|c| c.contains("allow 22/tcp")),
        "{:?}",
        calls
    );
    assert!(calls.iter().any(|c| c.contains("status")), "{:?}", calls);
    assert!(
        calls.iter().any(|c| c.contains("delete allow 22/tcp")),
        "{:?}",
        calls
    );
    // And the backend still answers to the name a line is written with.
    assert_eq!(caps.name(), "firewall");
}
