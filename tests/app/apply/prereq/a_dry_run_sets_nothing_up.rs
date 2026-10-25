use super::*;

/// A preview performs nothing — the property this repo's flagship bug broke. `--yes` is
/// set too, so the only thing stopping it is the dry-run check.
#[tokio::test]
async fn a_dry_run_sets_nothing_up() {
    let mut h = harness(&[("mix hex.info", false, "could not be found")]);
    h.config = Arc::new(shall::config::Config {
        yes: true,
        dry_run: true,
        ..Default::default()
    });
    h.prereqs()
        .offer_row(&builtin("mix"), "mix", &[String::new()])
        .await;
    assert!(
        !h.ran("mix local.hex --force").await,
        "a preview installed something"
    );
}
