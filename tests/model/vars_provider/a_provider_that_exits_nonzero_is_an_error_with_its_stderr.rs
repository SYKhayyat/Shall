use super::*;

#[test]
fn a_provider_that_exits_nonzero_is_an_error_with_its_stderr() {
    let tmp = tmp();
    let dir = tmp.path();
    let (name, body) = if cfg!(windows) {
        ("vars.cmd", "@echo off\r\necho boom 1>&2\r\nexit /b 3\r\n")
    } else {
        ("vars.sh", "echo boom >&2\nexit 3\n")
    };
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    let err = run_external(&path, &facts()).unwrap_err();
    assert!(
        err.what.contains("boom"),
        "carries the provider's stderr: {}",
        err
    );
}
