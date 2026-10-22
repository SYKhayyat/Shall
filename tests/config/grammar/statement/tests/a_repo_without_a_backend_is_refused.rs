use super::*;

#[test]
fn a_repo_without_a_backend_is_refused() {
    // A repository belongs to one package manager; guessing runs the wrong system
    // command (V.47). `snap` isn't in this test's known set, so it also proves the
    // backend is validated.
    let err = p("repo:ppa:deadsnakes/ppa").unwrap_err();
    assert!(err.what.contains("not a backend"), "{}", err);
    assert!(err.hint.unwrap().contains("apt"));
}
