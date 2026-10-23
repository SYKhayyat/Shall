use super::*;

/// `pacman -Ss`, `dnf search` and `brew search` all exit non-zero when the query
/// simply matched nothing. That is an answer, and must survive as one.
#[tokio::test]
async fn a_quiet_nonzero_exit_is_an_ordinary_empty_result() {
    let (e, _m) = exec("pacman -Ss nosuchpkg", DryRunOutput::faulted(""));
    let out = e
        .search_output("pacman", &["-Ss", "nosuchpkg"], false)
        .await
        .expect("an empty search is not a fault");
    assert!(out.is_empty());
}
