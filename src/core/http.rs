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
static BUILDS: Lazy<DashMap<String, usize>> = Lazy::new(DashMap::new);

/// A no-op in every build that is not a test, so the counter cannot be forgotten on the way out
/// and `build` reads the same in both.
fn count_build_for_tests(policy: &Policy) {
    *BUILDS.entry(policy.user_agent.clone()).or_insert(0) += 1;
}

/// **`pub` for the integration tests' sake.** The mirror links the library without `cfg(test)`
/// and needs to inspect the pool without reimplementing its policy key.
#[doc(hidden)]
pub fn pool_size_for_test(user_agent: &str) -> usize {
    POOL.iter()
        .filter(|entry| entry.key().user_agent == user_agent)
        .count()
}

/// **`pub` for the integration tests' sake.** The concurrent-build assertion needs the same
/// process-wide counter that the client builder updates; duplicating it would test nothing.
#[doc(hidden)]
pub fn builds_for_test(user_agent: &str) -> usize {
    BUILDS.get(user_agent).map(|count| *count).unwrap_or(0)
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
