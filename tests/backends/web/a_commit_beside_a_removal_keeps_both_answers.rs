use super::*;

/// **A removal that fails leaves the record alone, even while another task is committing.**
///
/// The other half of the same merge. `remove` used to put the entry back into its *local*
/// copy and then write that copy whole; now it simply never names the url as removed, and
/// the shared record stands. A merge that dropped it anyway would make a still-installed
/// package invisible to the next sync — drift nothing can see, which is worse than the
/// failure it came from.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_commit_beside_a_removal_keeps_both_answers() {
    let (web, _mock, _tmp) = backend("race2");
    let core = web.core.clone();
    record(
        &core,
        "https://example.test/kept",
        handed_to("dpkg", "kept", "https://example.test/kept"),
    )
    .await;
    record(
        &core,
        "https://example.test/gone",
        handed_to("dpkg", "gone", "https://example.test/gone"),
    )
    .await;

    let remover = {
        let core = core.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
            core.commit_state(Vec::new(), vec!["https://example.test/gone".into()])
                .await
                .expect("the removal commits");
        })
    };
    let installer = {
        let core = core.clone();
        tokio::spawn(async move {
            core.commit_state(
                vec![(
                    "https://example.test/new".into(),
                    handed_to("dpkg", "new", "https://example.test/new"),
                )],
                Vec::new(),
            )
            .await
            .expect("the install commits");
        })
    };
    remover.await.expect("remover");
    installer.await.expect("installer");

    let state = core.load_state().await.unwrap();
    assert!(
        state.contains_key("https://example.test/kept"),
        "an untouched record vanished"
    );
    assert!(
        state.contains_key("https://example.test/new"),
        "the concurrent install was lost"
    );
    assert!(
        !state.contains_key("https://example.test/gone"),
        "the removal was undone by the concurrent install's write — which is the lost update \
         in the other direction, and it leaves a record for a file that is gone"
    );
}
