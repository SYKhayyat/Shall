use super::*;

/// The reported case: mix cannot install anything, and `--yes` is the flag that says do it
/// (owner ruling, 2026-07-29). Measured in the tools image: without Hex, `mix hex.info` is
/// `** (Mix) The task "hex.info" could not be found`, exit 1.
#[tokio::test]
async fn an_unmet_prerequisite_is_set_up_under_yes() {
    let mut h = harness(&[(
        "mix hex.info",
        false,
        "** (Mix) The task \"hex.info\" could not be found",
    )]);
    h.config = Arc::new(shall::config::Config {
        yes: true,
        ..Default::default()
    });
    h.prereqs()
        .offer_row(&builtin("mix"), "mix", &[String::new()])
        .await;
    assert!(
        h.ran("mix local.hex --force").await,
        "the setup command never ran: {:?}",
        h.mock.get_calls().await
    );
}
