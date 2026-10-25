use super::*;

/// `brew info` answers about the formula, not about this machine — `"installed": []` was
/// the only thing distinguishing the two, and it was read for its properties while its
/// emptiness was ignored. So `info` said `Some` for every formula Homebrew has heard of.
#[tokio::test]
async fn info_answers_installed_here_not_known_to_a_tap() {
    let (mock, core) = mocked();
    mock.set_response(
        "brew info --json=v1 -- jq",
        Ok(shall::core::executor::DryRunOutput {
            stdout: brew_info_json("[]").into_bytes(),
            stderr: vec![],
        }
        .into()),
    );
    assert!(
        BrewQueryable { core }.info("jq").await.unwrap().is_none(),
        "a formula a tap knows was reported as installed — `install` then does nothing"
    );
}
