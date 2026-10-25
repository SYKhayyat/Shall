use super::*;

/// The disk layer's tests share one `SHALL_DATA_DIR`, so they run as one test rather than
/// racing each other over the same directory. The suite-wide lock is taken too: `datalock`'s
/// tests repoint the same process-global, and without it whichever test set the variable
/// second sent this one reading and writing some other directory — which surfaced as a
/// flake only under full-suite load.
#[test]
fn a_listing_survives_a_run_only_when_asked_for_and_only_until_it_is_stale() {
    let _env = shall::core::shall_data_dir_lock();
    let dir = std::env::temp_dir().join(format!("shall-cache-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::env::set_var("SHALL_DATA_DIR", &dir);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        // Off by default: a second "run" asks the manager again.
        let calls = AtomicUsize::new(0);
        for _ in 0..2 {
            let run = InstalledListings::new();
            run.once("apt", async {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(vec![pkg("jq")])
            })
            .await
            .unwrap();
        }
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "a listing crossed runs with the cache off — the default must never persist"
        );

        // On: the second run answers from disk without reaching the manager.
        let calls = AtomicUsize::new(0);
        let mut seen: std::sync::Arc<Vec<Package>> = Default::default();
        for _ in 0..3 {
            let run = InstalledListings::with_ttl(600);
            seen = run
                .once("apt", async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![pkg("jq"), pkg("ripgrep")])
                })
                .await
                .unwrap();
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1, "three runs, one listing");
        assert_eq!(seen.len(), 2, "the cached answer must be the whole answer");
        assert_eq!(seen[0].name, "jq");

        // A TTL of one second, with the entry written far enough in the past to be stale:
        // expiry is what bounds how wrong a cached machine can be.
        let stale = InstalledListings::with_ttl(1);
        let path = InstalledListings::cache_file("apt").unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let mut entry: CachedListing = serde_json::from_str(&raw).unwrap();
        entry.taken_at -= 60;
        std::fs::write(&path, serde_json::to_string(&entry).unwrap()).unwrap();
        let refetched = AtomicUsize::new(0);
        stale
            .once("apt", async {
                refetched.fetch_add(1, Ordering::SeqCst);
                Ok(vec![pkg("jq")])
            })
            .await
            .unwrap();
        assert_eq!(
            refetched.load(Ordering::SeqCst),
            1,
            "an expired listing was served — the TTL bounds nothing"
        );

        // A mutation drops the disk layer too, not just the memo. A `forget_all` that cleared
        // one and left the other would re-read the pre-mutation answer immediately.
        let mutating = InstalledListings::with_ttl(600);
        mutating
            .once("apt", async { Ok(vec![pkg("jq")]) })
            .await
            .unwrap();
        assert!(InstalledListings::cache_file("apt").unwrap().exists());
        mutating.forget_all();
        assert!(
            !InstalledListings::cache_file("apt").unwrap().exists(),
            "the memo was forgotten and the file that would answer the next question was not"
        );

        // A backend name can never escape the cache directory.
        let escaped = InstalledListings::cache_file("../../evil").unwrap();
        assert_eq!(
            escaped.parent(),
            Some(InstalledListings::cache_dir()).as_deref()
        );

        // Unreadable is a miss, never a failure and never an empty machine.
        let corrupt = InstalledListings::with_ttl(600);
        std::fs::create_dir_all(InstalledListings::cache_dir()).unwrap();
        std::fs::write(InstalledListings::cache_file("npm").unwrap(), "{ not json").unwrap();
        let got = corrupt
            .once("npm", async { Ok(vec![pkg("prettier")]) })
            .await
            .unwrap();
        assert_eq!(got.len(), 1, "a corrupt cache file must read as a miss");

        std::env::remove_var("SHALL_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    });
}
