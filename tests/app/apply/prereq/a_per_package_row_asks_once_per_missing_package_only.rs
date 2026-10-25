use super::*;

/// asdf's prerequisite is per declared tool, and the probe answers for all of them at
/// once. `asdf plugin list` exits 0 whatever it says, so a plugin that IS there must be
/// read out of the output — and one that is not must still be offered.
#[tokio::test]
async fn a_per_package_row_asks_once_per_missing_package_only() {
    let mut h = harness(&[("asdf plugin list", true, "jq\n")]);
    h.config = Arc::new(shall::config::Config {
        yes: true,
        ..Default::default()
    });
    h.prereqs()
        .offer_row(
            &builtin("asdf"),
            "asdf",
            &["jq".to_string(), "nodejs".to_string()],
        )
        .await;
    assert!(
        h.ran("asdf plugin add nodejs").await,
        "the missing plugin was not offered: {:?}",
        h.mock.get_calls().await
    );
    assert!(
        !h.ran("asdf plugin add jq").await,
        "it added a plugin that `asdf plugin list` already showed"
    );
    assert_eq!(
        h.mock
            .get_calls()
            .await
            .iter()
            .filter(|c| c.as_str() == "asdf plugin list")
            .count(),
        1,
        "the probe ran once for the whole manager, not once per declared tool"
    );
}
