use super::*;

/// **The invariant that matters most here, and the one nothing was checking.**
///
/// When the system manager refuses, the files are still on disk and still on PATH. Dropping
/// the record anyway would make the resource drift *no sync can see*: Shall would have
/// forgotten it, so nothing would ever try again and nothing would report it. The record
/// goes back, and the call returns an error naming what is still installed.
#[tokio::test]
async fn a_failed_removal_keeps_the_record_rather_than_forgetting_the_resource() {
    let (web, mock, _tmp) = backend("stuck");
    let url = "https://example.invalid/fd_10.2.0_amd64.deb";
    record(&web.core, url, handed_to("dpkg", "fd", url)).await;

    mock.set_response(
        &removal_argv("dpkg -r fd"),
        Err(Error::Other("dpkg: dependency problems".into())),
    );
    let err = web
        .remove(&[url.to_string()], false, reaped().await)
        .await
        .expect_err("a manager that refuses must not read as a removal");
    assert!(
        err.to_string().contains("still on disk"),
        "the error does not say the resource is still installed: {err}"
    );
    assert!(
        web.core.load_state().await.unwrap().contains_key(url),
        "the record was dropped for a resource that is still installed — the one state no \
         sync can detect"
    );
}
