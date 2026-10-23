use super::*;

/// The boundary, and it was drawn in the wrong place first.
///
/// A read that failed and *complained* is left alone here. It looks like it ought to be a
/// failure — `search_output` treats exactly that shape as one — but a listing is not a
/// search, and the caller may have a reading for it that this primitive cannot know.
/// Making it fatal failed a whole `sync`: `Get-ComputerRestorePoint` on an unelevated
/// shell exits 1 with `Access denied` on stderr and nothing on stdout, and the snapshot
/// check that asked has every right to carry on without a restore-point list.
///
/// Whether *that* caller should care is a real question and a separate one. The rule
/// here is only about what a command said: silence on both streams has one reading,
/// and a complaint has two.
#[tokio::test]
async fn a_read_that_failed_but_explained_itself_is_left_to_its_caller() {
    let (e, mock) = wired();
    mock.set_response(
        "winget list",
        Ok(super::spoken_failure(1, "", "Failed when opening source")),
    );
    let out = e
        .run_output("winget", &["list"], false)
        .await
        .expect("a read that complained has said something; this primitive does not judge it");
    assert!(
        out.is_empty(),
        "the answer is still the empty stdout: {out:?}"
    );
}
