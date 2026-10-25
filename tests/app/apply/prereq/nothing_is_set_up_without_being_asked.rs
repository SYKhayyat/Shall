use super::*;

/// And without it. A test's stdin is not a terminal, so this is also the unattended path:
/// Shall says what it would have asked and changes nothing.
#[tokio::test]
async fn nothing_is_set_up_without_being_asked() {
    let h = harness(&[("mix hex.info", false, "could not be found")]);
    h.prereqs()
        .offer_row(&builtin("mix"), "mix", &[String::new()])
        .await;
    assert!(
        !h.ran("mix local.hex --force").await,
        "it ran a setup command nobody agreed to"
    );
}
