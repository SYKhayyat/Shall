use super::*;

/// The rule is the engine's, not chocolatey's, so it is asserted on every policy that can
/// reach the state — an absent phrasing printed while the index was unreachable.
///
/// apt is the one that bites in the field: a sources.list it could not fetch makes
/// `Unable to locate package` the answer for packages that plainly exist.
///
/// Each string here is that policy's *own* two markers put in one output. They assert the
/// precedence rule, not that any manager phrases a run exactly this way — the wording of
/// each half was measured when the marker was added, and choco's pair is a captured run
/// (`install-source-unreachable.txt`) rather than a composition.
#[test]
fn no_manager_calls_a_name_absent_while_it_is_also_reporting_a_transient_failure() {
    let cases = [
        (
            apt(),
            "E: Failed to fetch http://deb.debian.org/\nE: Unable to locate package jq",
        ),
        (
            dnf(),
            "Cannot download repomd.xml\nNo match for argument: jq",
        ),
        (
            pacman(),
            "error: failed retrieving file\nerror: target not found: jq",
        ),
        (
            apk(),
            "ERROR: temporary error (try again later)\nERROR: unable to select packages:",
        ),
        (
            brew(),
            "curl: (28) Operation timed out\nNo available formula with the name \"jq\"",
        ),
        (
            cargo(),
            "spurious network error\ncould not find `jq` in registry",
        ),
        (pixi(), "failed to fetch\nNo candidates were found for jq"),
    ];
    for (policy, output) in cases {
        assert!(
            !policy.names_an_absent_package(&ExitPolicy::haystack(output.as_bytes(), b"")),
            "a name was called absent although the same output says the fetch failed: {output:?}"
        );
        assert_eq!(
            policy.retryability(&ExitPolicy::haystack(output.as_bytes(), b"")),
            Retryability::Transient,
            "{output:?}"
        );
    }
}
