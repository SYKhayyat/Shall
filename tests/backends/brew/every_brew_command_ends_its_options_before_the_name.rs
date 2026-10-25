use super::*;

#[tokio::test]
async fn every_brew_command_ends_its_options_before_the_name() {
    let (mock, core) = mocked();
    let spec = PackageSpec {
        name: "ripgrep".into(),
        backend: "brew".into(),
        ..Default::default()
    };
    BrewInstallable { core: core.clone() }
        .install(&[spec], false)
        .await
        .unwrap();
    BrewInstallable { core: core.clone() }
        .remove(
            &["ripgrep".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();
    BrewQueryable { core: core.clone() }
        .info("ripgrep")
        .await
        .ok();
    BrewSearchable { core: core.clone() }
        .search("ripgrep")
        .await
        .unwrap();
    core.get_dependencies("ripgrep").await.unwrap();

    let calls = mock.get_calls().await;
    assert_eq!(
        calls,
        vec![
            "brew install -- ripgrep",
            "brew uninstall -- ripgrep",
            "brew info --json=v1 -- ripgrep",
            "brew search -- ripgrep",
            "brew deps -- ripgrep",
        ]
    );
}
