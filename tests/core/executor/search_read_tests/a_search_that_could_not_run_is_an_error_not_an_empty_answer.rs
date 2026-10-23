use super::*;

/// Every manager that can be first in `priority` reaches this through its own
/// `search`; the rule has to hold for all of them, so it is tested here once.
#[tokio::test]
async fn a_search_that_could_not_run_is_an_error_not_an_empty_answer() {
    let (e, _m) = exec(
        "apt-cache search jq",
        DryRunOutput::faulted("E: The package lists are empty."),
    );
    let err = e
        .search_output("apt-cache", &["search", "jq"], false)
        .await
        .unwrap_err()
        .to_string();
    assert!(err.contains("could not answer"), "{}", err);
    assert!(err.contains("package lists are empty"), "{}", err);
}
