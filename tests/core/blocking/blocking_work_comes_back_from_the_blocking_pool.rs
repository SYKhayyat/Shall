use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn blocking_work_comes_back_from_the_blocking_pool() {
    assert_eq!(off_the_runtime(|| 6 * 7).await.expect("no panic"), 42);
}
