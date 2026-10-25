use super::*;

/// The half that keeps the offer from becoming noise: Hex is already there, so nothing is
/// printed, nothing is asked, and nothing runs. Without this, every `mix` sync on every
/// machine would offer to install a thing that is installed.
#[tokio::test]
async fn a_prerequisite_that_is_already_met_runs_nothing() {
    // `yes` is set on purpose. Without it this passes whatever the probe decides, because
    // an unconfirmed offer runs nothing either — the check would be testing the
    // confirmation and claiming to test the probe. Measured: with `yes` off, deleting the
    // satisfied-check leaves this green.
    let mut h = harness(&[("mix hex.info", true, "Hex: 2.5.1")]);
    h.config = Arc::new(shall::config::Config {
        yes: true,
        ..Default::default()
    });
    h.prereqs()
        .offer_row(&builtin("mix"), "mix", &[String::new()])
        .await;
    assert!(
        !h.ran("mix local.hex --force").await,
        "it set up something that was already set up"
    );
}
