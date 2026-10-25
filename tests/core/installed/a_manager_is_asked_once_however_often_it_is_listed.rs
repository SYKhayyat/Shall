use super::*;

#[tokio::test]
async fn a_manager_is_asked_once_however_often_it_is_listed() {
    let memo = InstalledListings::new();
    let calls = AtomicUsize::new(0);

    for _ in 0..25 {
        let got = memo
            .once("apt", async {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(vec![pkg("jq")])
            })
            .await
            .unwrap();
        assert_eq!(got.len(), 1);
    }

    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "twenty-five listings reached the manager — this is the `declared + 1` shape the \
         memo exists to remove"
    );
}
