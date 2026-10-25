use super::*;

/// **Two concurrent installs both keep their record.**
///
/// The lost-update race this closes: `load_state` took the internal lock, read the whole
/// file and released it; `save_state` took it again and wrote the whole file. In `install`
/// those two calls are 220 lines and a download apart, so two `web:` packages in one wave
/// both read `{}` and whichever saved last wrote a map holding only its own record. The
/// record that lost describes a deployed file and its `bin_link`: the package then reads as
/// unmanaged, the next sync cannot see what it deployed, and teardown has nothing to remove.
///
/// **What makes this reproduce, where nothing in the suite did.** `MockExecutor` has had a
/// `delays` map since it was written — *"how long a command takes, for tests about
/// concurrency rather than about output"* — and it was used on exactly two commands, both
/// *reads*, in one file about graph ordering. No mutating command anywhere was ever given a
/// non-zero duration, which is precisely why R1, R2 and R4 could all be real and never fire:
/// mocked, the "download" returns in microseconds and the window is essentially closed. The
/// yields below are that missing duration, at the one point that matters — between reading
/// the state and committing to it.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_concurrent_installs_do_not_lose_each_other() {
    let (web, _mock, _tmp) = backend("race");
    let core = web.core.clone();

    // Both tasks read first, both then wait, and only then does either commit — which is
    // the interleaving `install` produces when two packages download at once. Under the old
    // read-modify-write this is the exact sequence that loses A.
    let a = {
        let core = core.clone();
        tokio::spawn(async move {
            let _seen = core.load_state().await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(60)).await;
            core.commit_state(
                vec![(
                    "https://example.test/a".into(),
                    handed_to("dpkg", "a", "https://example.test/a"),
                )],
                Vec::new(),
            )
            .await
            .expect("A commits");
        })
    };
    let b = {
        let core = core.clone();
        tokio::spawn(async move {
            let _seen = core.load_state().await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            core.commit_state(
                vec![(
                    "https://example.test/b".into(),
                    handed_to("dpkg", "b", "https://example.test/b"),
                )],
                Vec::new(),
            )
            .await
            .expect("B commits");
        })
    };
    a.await.expect("A");
    b.await.expect("B");

    let state = core.load_state().await.unwrap();
    assert!(
        state.contains_key("https://example.test/a"),
        "A's record was overwritten by B's whole-map write — the read-modify-write is back. \
         The state must be merged under one lock, not rewritten from what each task read \
         before its download."
    );
    assert!(
        state.contains_key("https://example.test/b"),
        "B's record is missing, which is the same defect with the tasks the other way round"
    );

    // And on disk, not only in the memo: a map that is right in memory and wrong in the
    // file is the same bug one process later.
    let on_disk = std::fs::read_to_string(&core.state_file).expect("the state file");
    assert!(
        on_disk.contains("/a") && on_disk.contains("/b"),
        "the file holds only one of the two records:\n{on_disk}"
    );
    assert!(
        !on_disk.contains("\n  "),
        "the state file is pretty-printed again. `core::state` ruled this for the registry \
         beside it — machine-read bookkeeping nobody opens, and pretty printing roughly \
         doubles the bytes."
    );
}
