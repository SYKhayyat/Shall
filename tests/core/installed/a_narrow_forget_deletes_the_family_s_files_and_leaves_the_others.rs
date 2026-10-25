use super::*;

/// **The disk layer goes with it, per backend.** The memo files are one per backend, so a
/// narrow forget removes exactly the files of the lock family it dropped — and Y6's ruling
/// that "any mutation drops it, on disk as well as in memory" is why the narrow path has to
/// delete rather than merely skip the write.
/// **Sync, holding the suite-wide `SHALL_DATA_DIR` lock**, for the reason the disk test
/// above spells out: this repoints a process-global and then awaits, so without the lock
/// another test that repoints the same variable in that window sends the assertions below to
/// a directory nothing was written to. It surfaced as `pacman wrote no cache file to
/// forget`, under full-suite load and nowhere else. `#[tokio::test]` cannot take the lock —
/// that is a `std::sync` guard held across an await — so the runtime is built by hand, which
/// is what the disk test above does too.
#[test]
fn a_narrow_forget_deletes_the_family_s_files_and_leaves_the_others() {
    let _env = shall::core::shall_data_dir_lock();
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("SHALL_DATA_DIR", dir.path());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let memo = InstalledListings::with_ttl(600);

        let fetch = |name: &'static str| async move { Ok(vec![pkg(name)]) };
        for backend in ["pacman", "yay", "brew"] {
            memo.once(backend, fetch(backend)).await.expect("listing");
        }
        let file = |backend: &str| InstalledListings::cache_file(backend).expect("a cache file");
        for backend in ["pacman", "yay", "brew"] {
            assert!(
                file(backend).exists(),
                "{backend} wrote no cache file to forget"
            );
        }

        memo.forget_manager("pacman");

        for backend in ["pacman", "yay"] {
            assert!(
                !file(backend).exists(),
                "{backend} shares `pacman`'s lock, so its cache file survived the narrow \
                 forget and the next run will read the pre-mutation answer straight off disk"
            );
        }
        assert!(
            file("brew").exists(),
            "the narrow forget deleted a cache file for a manager no mutation could have \
             changed"
        );
    });

    std::env::remove_var("SHALL_DATA_DIR");
}
