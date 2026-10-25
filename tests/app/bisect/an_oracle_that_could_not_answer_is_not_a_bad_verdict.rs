use super::*;

/// **A failing oracle stops the search rather than being read as a verdict.**
///
/// The real oracle restores a snapshot before it tests, so a restore that fails is not the
/// answer "bad" — it is no answer at all, and treating it as one would name an innocent
/// snapshot as the culprit. The shipped loop propagated that error; the tested one could
/// not express it, which is the second half of why one function is better than two.
#[tokio::test]
async fn an_oracle_that_could_not_answer_is_not_a_bad_verdict() {
    let err = first_bad(8, |_| async move {
        Err(shall::core::Error::Snapshot("restore failed".into()))
    })
    .await
    .expect_err("a failed restore must not be read as a failing test");
    assert!(err.to_string().contains("restore failed"));
}
