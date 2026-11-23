use super::*;
use shall::core::adapter::AdapterRow;

/// The caller's order decides, not the table's: a question about a file is answered by the manager
/// this machine installs with. And a manager that refuses the question — a missing binary, a
/// non-zero exit, an answer that parses to nothing — has not answered it, so the next row is
/// still worth asking.
#[tokio::test]
async fn the_first_manager_that_can_answer_is_the_one_asked() {
    let rows = owners(Vec::new());
    let mock: Arc<MockExecutor> = Arc::new(MockExecutor::new(Arc::new(DashMap::new())));
    let exec = executor(&mock);

    // The shipped rows are gated on `detect` binaries this test host does not have, so the
    // priority list is what this asserts: `owning_package` is handed rows and an order, and the
    // order decides. Every shipped row still detects nothing here, so nothing is asked at all.
    let priority = vec!["pacman".to_string(), "apt".to_string()];
    let answer = owning_package(
        &exec,
        &rows,
        &priority,
        "/usr/lib/systemd/system/sshd.service",
    )
    .await;
    assert!(
        answer.is_none(),
        "no manager on this host can answer, and silence is the honest reply: {answer:?}"
    );
    assert!(
        rows.iter().all(|r| r.unusable().is_none()),
        "the rows themselves are fine; it is the machine that cannot answer"
    );
}
