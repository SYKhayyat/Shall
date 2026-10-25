use super::*;

/// The cache is bounded, and the bound is reachable.
///
/// `watch` is the only caller that makes this process long-lived — a `sync` exits and takes
/// its cache with it — so the leak this bound prevents is invisible to every other command
/// and to every test that does not look for it.
#[test]
fn the_cache_stops_growing() {
    for i in 0..(CAPACITY + 50) {
        let _ = compiled(&format!(r"^shall-bound-test-{i}-(\d+)$"));
    }
    assert!(
        CACHE.len() <= CAPACITY,
        "the cache holds {} patterns against a ceiling of {}; a `watch` that runs for a \
         week accumulates one automaton per pattern its config has ever held",
        CACHE.len(),
        CAPACITY
    );
    // And it is still a cache afterwards: the ceiling must not turn it into a no-op, which
    // would trade a slow leak for recompiling on every line of every listing.
    let a = compiled(r"^shall-bound-still-caching-(\d+)$").unwrap();
    let b = compiled(r"^shall-bound-still-caching-(\d+)$").unwrap();
    assert!(
        Arc::ptr_eq(&a, &b),
        "past the ceiling the cache stopped caching"
    );
}
