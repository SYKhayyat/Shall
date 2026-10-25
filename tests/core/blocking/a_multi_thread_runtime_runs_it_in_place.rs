use super::*;

/// And the case it exists for: on a multi-thread runtime the work still runs, and returns.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_multi_thread_runtime_runs_it_in_place() {
    assert_eq!(on_the_terminal(|| "asked"), "asked");
}
