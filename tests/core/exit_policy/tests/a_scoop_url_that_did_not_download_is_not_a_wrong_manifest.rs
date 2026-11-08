//! **`URL … is not valid` is about the download, and the URL in it is usually fine.**
//!
//! scoop's `lib/download.ps1` prints that sentence inside the `catch` around its download call,
//! so it accompanies *whatever the exception was* — a 504 from the raw host, a reset connection,
//! a rate limit. The sentence names a URL as the suspect; the thing that failed was the transfer.
//!
//! The nightly of 2026-11-08 reds the Windows native sweep on `scoop:jq` with exactly this line
//! against `jqlang/jq`'s `jq-1.8.2` asset. That URL answers `200` from every machine that has
//! asked it since, and scoop's own published manifest carries a hash that matches it — so the
//! defect was not in the manifest and not in the URL. Classed `Permanent`, Shall told the user
//! `a defect, not ecosystem variance` about a manifest nobody had got wrong, and refused to
//! retry a fetch that would have cleared.
//!
//! **Both directions.** A transient reading that swallowed real manifest defects would be the
//! worse bug: `couldn't find manifest for 'x'` must stay permanent and absent, whatever else the
//! output says. The last case here is that one, run through the same policy that now carries the
//! new marker.
use super::*;

/// The sentence scoop prints when its download call raised, in the shape the Windows sweep saw.
const SCOOP_URL_NOT_VALID: &[u8] = b"ERROR URL \
https://github.com/jqlang/jq/releases/download/jq-1.8.2/jq-windows-amd64.exe#/jq.exe is not \
valid";

#[test]
fn a_scoop_url_that_did_not_download_is_retried_rather_than_called_a_defect() {
    let p = scoop();
    let verdict = p.retryability_of(Some(1), &ExitPolicy::haystack(SCOOP_URL_NOT_VALID, b""));
    assert!(
        p.signals_failure(&ExitPolicy::haystack(SCOOP_URL_NOT_VALID, b"")),
        "the sentence opens with scoop's own `ERROR ` prefix, so it is a failure; this test is \
         about which KIND, and a change that stopped seeing it at all would make the assertion \
         below vacuous"
    );
    assert_eq!(
        verdict,
        Retryability::Transient,
        "scoop wraps every download exception in this sentence, and the URL it names answered \
         200 — reading it as permanent reports a defect about a manifest that is correct and \
         refuses the retry that clears it"
    );
}

/// scoop's own `abort` line for the same case, printed alongside.
#[test]
fn a_scoop_download_failure_is_retried_rather_than_called_a_defect() {
    let p = scoop();
    assert_eq!(
        p.retryability_of(
            Some(1),
            &ExitPolicy::haystack(b"ERROR download failed. Use scoop to file an issue.", b"")
        ),
        Retryability::Transient,
        "the same `catch` prints this line, so it is the same fact as the sentence above"
    );
}

/// **A genuinely wrong name alone is still permanent**, through the policy that now carries the
/// widened transient list — the widening must not swallow the case the marker list exists for.
#[test]
fn a_missing_manifest_alone_is_still_permanent() {
    let p = scoop();
    let output = ExitPolicy::haystack(b"ERROR Couldn't find manifest for 'nope'.", b"");
    assert_eq!(
        p.retryability_of(Some(0), &output),
        Retryability::Permanent,
        "scoop exited 0 and said it could not find the manifest; that is a name in the wrong \
         place and no retry changes it"
    );
    assert!(
        p.names_an_absent_package(&output),
        "and it is still the manager saying the name is not in its index"
    );
}

/// **When both facts are in one breath, transient wins — and that is the documented order, not
/// an accident of marker order.** `retryability` consults `transient` before `absent` because an
/// absent verdict is a claim about an index, and a manager that reports the index as unreadable
/// in the same output has not earned it. What the widening must not do is turn a *clean* missing
/// manifest into a retry; the test above is that case, and this one is the other direction.
#[test]
fn a_missing_manifest_and_a_download_failure_together_still_retry() {
    let p = scoop();
    let output = ExitPolicy::haystack(
        b"ERROR Couldn't find manifest for 'nope'.",
        b"ERROR URL https://example.invalid/x is not valid",
    );
    assert_eq!(
        p.retryability_of(Some(1), &output),
        Retryability::Transient,
        "scoop reported the index as unreachable while also not finding the name, so its absent \
         claim is not one it has earned and the run retries"
    );
}
