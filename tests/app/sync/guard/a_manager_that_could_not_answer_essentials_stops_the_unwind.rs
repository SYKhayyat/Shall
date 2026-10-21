use super::*;

/// **A manager that could not answer the essential question refuses the unwind too.** This is
/// the other objection, and it is a different one: the package is not protected by any rule, but
/// Shall cannot find out whether the manager considers it essential, so it does not know what it
/// would be removing.
///
/// "Shall is not removing anything it owns" is the sentence, and the manager that could not
/// answer is named in it — which is the third mutation the gate reported as surviving, because
/// the `UnverifiedEssentials` arm is the only place that name is written.
#[tokio::test]
async fn a_manager_that_could_not_answer_essentials_stops_the_unwind() {
    // The third field is "the essential query fails".
    let reg = registry_with(&[("apt", &[], true)]);
    let cfg = config_with(0);
    let err = enforce_own_work(&cfg, &reg, &pairs(&["htop"]))
        .await
        .expect_err("an undo through a manager that cannot answer must be refused");
    let msg = err.to_string();
    assert!(
        msg.contains("apt"),
        "the refusal names the manager that could not answer: {msg}"
    );
    assert!(
        msg.contains("could not be asked what it considers essential"),
        "the refusal says why: {msg}"
    );
    assert!(
        msg.contains("not removing anything it owns"),
        "and says what it is doing instead: {msg}"
    );
}
