use super::*;

/// **One panic must not switch the cache off for the rest of the run.**
///
/// `help_text` used to answer `probe(…)` on a poisoned lock, which reads as prudence — a cache
/// nobody trusts, a fresh answer. But a `Mutex` is poisoned *permanently* by the one panic, so
/// that branch turned a single panic anywhere in the process into every later question about
/// every program spawning a new process, silently, which is exactly the cost the cache exists
/// to remove. `PLAN.md` #81.
///
/// The test poisons both locks for real — a panic while each is held, which is the only way a
/// `Mutex` gets poisoned — and then asks a question the cache can answer from memory. With the
/// old arms it spawns a process for a program that does not exist and answers `None`, so the
/// cached text is gone and the test is red.
#[test]
fn a_poisoned_cache_is_still_a_cache() {
    const PROGRAM: &str = "shall-help-poison-canary-zzz";
    let key = format!("{PROGRAM} ");
    // Seed the cache with an answer that no probe could produce, so "answered from memory" and
    // "probed and found nothing" cannot be confused.
    {
        let mut map = cache().lock().expect("the cache starts unpoisoned");
        map.entry(key.clone())
            .or_default()
            .lock()
            .expect("the slot starts unpoisoned")
            .clone_from(&Some(Some("cached, not probed".to_string())));
    }

    // Poison the per-key slot, then the map, each by panicking while the lock is held.
    let slot = {
        let map = cache().lock().expect("unpoisoned to take the slot out");
        map.get(&key).cloned().expect("the slot exists")
    };
    let _ = std::panic::catch_unwind(|| {
        let _held = slot.lock().expect("unpoisoned to hold it");
        panic!("poison the slot");
    });
    let _ = std::panic::catch_unwind(|| {
        let _held = cache().lock().expect("unpoisoned to hold the map");
        panic!("poison the map");
    });

    let answered = help_text(PROGRAM, &[]);
    assert_eq!(
        answered.as_deref(),
        Some("cached, not probed"),
        "a poisoned lock deleted the cache: the answer came from a fresh probe of a program \
         that does not exist, so the run silently lost the cache it had already paid for. \
         A `Mutex` stays poisoned for good after one panic, so this branch is permanent."
    );

    // Leave the process as we found it — the cache is process-wide and this suite is not.
    cache().lock().map(|mut m| m.remove(&key)).ok();
    let _ = std::panic::catch_unwind(|| {
        let _held = slot.lock();
        panic!("clear the poison");
    });
}
