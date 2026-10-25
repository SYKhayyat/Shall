use super::*;

/// A backend with no lock in the table is never waited on at all — `held_for` answers
/// `Free`, so the first poll ends it.
#[tokio::test]
async fn a_backend_with_no_lock_is_never_waited_on() {
    let out = wait_until_not_held("npm", std::time::Duration::from_secs(300), &|| false).await;
    assert!(matches!(out, Waited::Freed(_)), "{out:?}");
}
