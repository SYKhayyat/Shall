use super::*;

/// luarocks names the wrong cause, and the name it gives is the one that would be acted
/// on. Captured from luarocks 3.13.0 on 2026-07-28.
///
/// The last line is `No results matching query were found for Lua 5.5` — which reads as
/// "this rock does not exist for your Lua", a fact about the request that no retry
/// changes. It is not what happened. Three lines above it, all three manifest mirrors
/// failed to download; luarocks then searched an empty index and reported the empty
/// result. The rock exists and `manifest-5.5.zip` was served fine to `curl` at the same
/// moment.
///
/// Classifying on the earlier lines rather than the summary is the whole point: read as
/// permanent, a widening of `install`'s withdrawal rule would delete the user's
/// declaration for a package that is perfectly real, and the harness would hard-fail a
/// machine whose only problem is its downloader.
#[test]
fn a_luarocks_manifest_that_would_not_download_is_transient_not_a_missing_rock() {
    assert_eq!(
        luarocks().retryability(&ExitPolicy::haystack(
            b"",
            LUAROCKS_MANIFEST_UNREACHABLE.as_bytes()
        )),
        Retryability::Transient,
        "the summary line was believed over the three download failures above it"
    );
}
