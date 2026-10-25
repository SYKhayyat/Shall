use super::*;

/// And when nothing has it either, "no such package" would be a lie.
#[tokio::test]
async fn nothing_found_past_a_silent_manager_says_so() {
    let err = settle(
        &bare_jq(),
        registry(
            DryRunOutput::faulted("E: package lists are empty"),
            DryRunOutput::new().into(),
        ),
    )
    .await
    .unwrap_err()
    .to_string();
    assert!(err.contains("could not answer"), "{}", err);
    assert!(err.contains("`first`"), "{}", err);
    assert!(err.contains("not be a misspelling"), "{}", err);
}
