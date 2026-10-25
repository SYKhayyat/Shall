use super::*;

/// A script the plan promised but the machine no longer has is a refusal, not an empty
/// body: `sh -c ""` exits 0, and the run would be recorded as a success against the old
/// hash with nothing having happened.
#[tokio::test]
async fn a_script_that_vanished_between_plan_and_apply_is_an_error_not_a_success() {
    let f = fx();
    let gone = f._tmp.path().join("departed.sh");
    let err = f
        .execs
        .run_exec_script(&gone)
        .await
        .expect_err("a missing script must refuse");
    let msg = err.to_string();
    assert!(msg.contains("cannot read"), "{msg}");
    assert!(
        msg.contains("departed.sh"),
        "the error names the file: {msg}"
    );
}
