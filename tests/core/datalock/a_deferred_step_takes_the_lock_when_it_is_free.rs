use super::*;

/// The waiting sibling of the door above, and the one `LockScope::Deferred` uses. It differs
/// from `try_for_one_step` only in what it does when the lock is taken, so what has to be
/// pinned separately is that the free case still hands one out.
///
/// A sync test with its own runtime rather than `#[tokio::test]`: it repoints
/// `SHALL_DATA_DIR`, and the guards that keep that from racing the rest of the suite must
/// be held outside an `await`.
#[test]
fn a_deferred_step_takes_the_lock_when_it_is_free() {
    let _g = gate();
    let dir = tmp("for-one-step");
    let _data_dir = DataDir::at(&dir);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    rt.block_on(async {
        let taken = DataLock::for_one_step("sync").await.unwrap();
        assert!(held(), "the deferred step's lock was taken and not counted");
        assert!(
            lock_file_is_in(&dir),
            "the deferred step locked a directory other than the one safe_data_dir names, so two runs would each hold a lock nobody else sees"
        );
        drop(taken);
        assert!(!held());
    });
}
