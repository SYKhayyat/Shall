use super::*;

/// And the backend that keeps the database claims them however the answers are ordered.
/// Registration order is the order the crawl replies in, and which backend owns a
/// declaration must not depend on it.
#[tokio::test]
async fn the_database_owner_claims_the_name_whichever_client_answers_first() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    let found = adopter(alpm_registry(mock, ["yay", "paru", "pacman"]))
        .discover()
        .await
        .unwrap();
    let backends: Vec<&str> = found.adopt.iter().map(|p| p.backend.as_str()).collect();
    assert_eq!(backends, vec!["pacman", "pacman", "pacman"]);
}
