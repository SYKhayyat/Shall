//! One connection pool per policy, for the life of the process.
//!
//! A `reqwest::Client` **is** the connection pool, the TLS session cache and the resolver
//! cache. Eight places in this tree built one per request, so every OSV advisory GET, every
//! registry query and every asset download paid a full TCP handshake, a full TLS handshake and
//! a re-parse of the root store — to a host the previous request had just finished talking to.
//! Clients built here are cached by the policy that distinguishes them, and cloning one is an
//! `Arc` bump, so a caller that asks per request still gets the same pool.
//!
//! The key is the whole policy and not just the user agent: a client that would follow an
//! HTTPS→HTTP redirect must never be handed to a caller that refuses one (SEC2), and a client
//! with no timeout must never be handed to an API call.

use crate::core::{Error, Result};
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::time::Duration;

/// Everything that makes two clients not interchangeable.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Policy {
    user_agent: String,
    /// `true` follows a redirect that leaves HTTPS. Only downloads that carry an explicit
    /// `@allow_http` may say true (SEC2).
    allow_downgrade: bool,
    /// `0` means no whole-request timeout, which is the only correct answer for a
    /// multi-gigabyte release asset and the wrong one for a JSON API call.
    timeout_secs: u64,
}

static POOL: Lazy<DashMap<Policy, reqwest::Client>> = Lazy::new(DashMap::new);

/// The shared client for this policy, building it the first time it is asked for.
pub fn client(
    user_agent: &str,
    allow_downgrade: bool,
    timeout_secs: u64,
) -> Result<reqwest::Client> {
    let key = Policy {
        user_agent: user_agent.to_string(),
        allow_downgrade,
        timeout_secs,
    };
    // **An entry, not a `get` and then an `insert`.** The two-step version has a window the width
    // of `build()` — which is not instantaneous, because a `reqwest::Client` builds a TLS
    // configuration — and every task that walks through it builds a client the pool then throws
    // away. Sixteen concurrent asks for one policy built up to sixteen clients and kept one, and
    // a discarded client is a discarded connection pool, so the cost lands on the first real
    // request rather than on the ask that caused it (`PLAN.md` #85).
    //
    // `entry` closes the window by holding the key's slot across the build, so the second caller
    // finds the first one's client instead of making its own. The build's error is raised rather
    // than swallowed, which is why this is a `match` on the two arms rather than
    // `or_insert_with`: `build` can fail, and a closure that cannot fail would have to invent an
    // answer for it.
    match POOL.entry(key) {
        Entry::Occupied(occupied) => Ok(occupied.get().clone()),
        Entry::Vacant(vacant) => {
            let built = build(vacant.key())?;
            vacant.insert(built.clone());
            Ok(built)
        }
    }
}

/// A pooled client for an API call: refuses a scheme downgrade, bounded by the configured
/// network timeout.
pub fn api(user_agent: &str, timeout_secs: u64) -> Result<reqwest::Client> {
    // A literal 0 reaches reqwest as "time out after zero seconds" — every request fails
    // instantly — rather than "no timeout", so an API caller's 0 is raised to 1 second here
    // and a caller that genuinely wants no bound asks `client` directly.
    client(user_agent, false, timeout_secs.max(1))
}

/// Counts every client this process actually builds, **keyed by user agent**, for the test that
/// drives `client` from sixteen threads at once.
///
/// Keyed, not a single number, for the reason `mine()` gives below: `cargo test` runs these
/// tests in parallel threads on one process, so a whole-process counter measures whatever the
/// four other tests happened to be building while this one ran — which is how the first version of
/// this test failed with the fix already in it. Test-only because a counter in the hot path is a
/// cost nobody asked for, and the only reader is the test.
#[cfg(test)]
static BUILDS: Lazy<DashMap<String, usize>> = Lazy::new(DashMap::new);

/// A no-op in every build that is not a test, so the counter cannot be forgotten on the way out
/// and `build` reads the same in both.
#[cfg(not(test))]
fn count_build_for_tests(_policy: &Policy) {}

#[cfg(test)]
fn count_build_for_tests(policy: &Policy) {
    *BUILDS.entry(policy.user_agent.clone()).or_insert(0) += 1;
}

fn build(policy: &Policy) -> Result<reqwest::Client> {
    let redirect = if policy.allow_downgrade {
        reqwest::redirect::Policy::default()
    } else {
        // The binding requirement is that the *final* download is HTTPS; checking each hop is
        // the cheapest correct form and also catches a downgrade in the middle of a chain that
        // ends back on HTTPS.
        reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().scheme() != "https" {
                return attempt.error("redirected to a non-HTTPS URL");
            }
            if attempt.previous().len() >= 10 {
                // `stop()` hands the last response back as content — a 3xx the caller would
                // read as the artifact. Too many hops is a failure, and it says so.
                return attempt.error("more than 10 redirects");
            }
            attempt.follow()
        })
    };
    let mut builder = reqwest::Client::builder()
        .user_agent(policy.user_agent.clone())
        .redirect(redirect);
    if policy.timeout_secs > 0 {
        builder = builder.timeout(Duration::from_secs(policy.timeout_secs));
    }
    count_build_for_tests(policy);
    builder.build().map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many clients this test's own policies hold. The pool is process-wide and the suite
    /// is concurrent, so a test that read `POOL.len()` as a whole would be measuring whatever
    /// other tests happened to be doing — which is exactly how these four came out flaky.
    fn mine(user_agent: &str) -> usize {
        POOL.iter()
            .filter(|e| e.key().user_agent == user_agent)
            .count()
    }

    #[test]
    fn asking_twice_for_one_policy_builds_one_client() {
        for _ in 0..20 {
            let _ = client("shall-pool-a", false, 15).unwrap();
        }
        assert_eq!(
            mine("shall-pool-a"),
            1,
            "twenty asks for one policy built more than one client — the pool is not pooling"
        );
    }

    #[test]
    fn a_client_that_would_follow_a_downgrade_is_never_handed_to_one_that_refuses() {
        let _strict = client("shall-pool-b", false, 15).unwrap();
        let _loose = client("shall-pool-b", true, 15).unwrap();
        assert_eq!(
            mine("shall-pool-b"),
            2,
            "the two redirect policies collapsed into one client — SEC2 would be enforced by \
             whichever caller happened to ask first"
        );
    }

    /// **The check-then-insert window, driven from sixteen threads at once.**
    ///
    /// The four tests above ask for one policy in a loop, in one thread, which is why the race
    /// survived all of them: a sequential caller always finds the client the last iteration left,
    /// and the pool's *contents* look identical however many clients were built and thrown away.
    /// So this one counts builds — the only observable that moves — and synchronises the callers
    /// on a barrier so they arrive together rather than in a queue.
    ///
    /// Sixteen is not decoration. Two threads can pass the `get` before either `insert`s often
    /// enough to matter; the point of a barrier is that none of them has started yet.
    #[test]
    fn sixteen_threads_asking_at_once_build_one_client() {
        use std::sync::{Arc, Barrier};
        const THREADS: usize = 16;
        // A fresh user agent per round, so each round is its own key and cannot be answered by
        // a client an earlier round left behind.
        let agent = format!("shall-pool-race-{}", std::process::id());
        let barrier = Arc::new(Barrier::new(THREADS));

        std::thread::scope(|scope| {
            for _ in 0..THREADS {
                let barrier = Arc::clone(&barrier);
                let agent = agent.clone();
                scope.spawn(move || {
                    barrier.wait();
                    let _ = client(&agent, false, 15).unwrap();
                });
            }
        });

        let built = BUILDS.get(&agent).map(|n| *n).unwrap_or(0);
        assert_eq!(
            built, 1,
            "{THREADS} threads asking for one policy at the same instant built {built} clients. \
             The pool keeps one and discards the rest, and a discarded client is a discarded \
             connection pool — the cost lands on the first real request, not on the ask that \
             caused it. `client` must go through `POOL.entry` (`PLAN.md` #85)."
        );
        assert_eq!(
            mine(&agent),
            1,
            "and the pool should hold exactly one client for that policy"
        );
    }

    #[test]
    fn the_timeout_is_part_of_the_key() {
        let _bounded = client("shall-pool-c", false, 15).unwrap();
        let _unbounded = client("shall-pool-c", false, 0).unwrap();
        assert_eq!(mine("shall-pool-c"), 2);
    }

    #[test]
    fn an_api_client_never_asks_reqwest_for_a_zero_second_timeout() {
        // reqwest reads a zero-second timeout as "fail instantly", not "no bound", so an API
        // caller handed a configured 0 must not pass it through.
        let _ = api("shall-pool-d", 0).unwrap();
        let _ = api("shall-pool-d", 1).unwrap();
        assert_eq!(
            mine("shall-pool-d"),
            1,
            "a 0-second API timeout was not raised to 1 — it built a distinct, \
             instantly-failing client"
        );
    }
}
