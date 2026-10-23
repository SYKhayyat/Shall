use super::*;

/// A read that failed **without a word** is not a machine with nothing on it.
///
/// Measured, on a real host: 3 of 16 cold-start concurrent `winget list` exit `0x8A150001`
/// having written zero bytes to either stream. `run_output` did not look at the status, so
/// the caller got `Ok("")`, the parser found no packages in it, and `list_installed`
/// answered `Ok(vec![])`. Nothing anywhere thought winget had failed — Shall believed the
/// machine was empty and said so at exit 0:
///
/// ```text
/// round 1 : rows min=0 max=280   EMPTY_LISTINGS=1/16
///         rc=0  ms=2285  rows=0   <-- `shall list --backend winget` reported no packages
/// ```
///
/// A silent non-zero exit is the one case that cannot be an answer: every manager with
/// nothing to report says so by exiting 0, or by printing a header. Saying nothing *and*
/// failing is the absence of an answer, and the caller has to be told which it got.
#[tokio::test]
async fn a_read_that_failed_without_a_word_is_not_an_empty_machine() {
    let (e, mock) = wired();
    mock.set_response("winget list", Ok(super::silent_failure(1)));
    let err = e
        .run_output("winget", &["list"], false)
        .await
        .expect_err("a silent non-zero read must not read as an empty listing");
    let msg = err.to_string();
    assert!(
        msg.contains("winget"),
        "the failure must name the command that produced nothing: {msg}"
    );
    assert!(
        msg.contains("no output"),
        "the failure must say the command produced nothing, so it cannot be \
         mistaken for an empty result: {msg}"
    );
}
