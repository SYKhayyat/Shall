use super::*;

/// The other half, and the reason this is not simply "non-zero reads are errors". A read
/// that exited non-zero and *printed a listing* has answered; the exit code is the manager
/// grumbling about something else. `apt list --installed` warning about an unstable CLI is
/// the shape.
#[tokio::test]
async fn a_read_that_failed_but_still_answered_keeps_its_answer() {
    let (e, mock) = wired();
    mock.set_response(
        "winget list",
        Ok(super::spoken_failure(
            1,
            "7zip.7zip  25.01
",
            "a warning",
        )),
    );
    let out = e
        .run_output("winget", &["list"], false)
        .await
        .expect("a read with a listing in it is an answer, whatever the exit code");
    assert!(out.contains("7zip.7zip"), "{out}");
}
