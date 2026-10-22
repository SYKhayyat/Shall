use super::*;

/// The two flags do not travel together (SEC2). helm downloads a plugin over HTTPS from a
/// git host; `--plain-http` is for OCI registries Shall does not address, so accepting
/// `@unverified` there did not also make `@allow_http` mean anything.
#[test]
fn allow_http_did_not_follow_unverified_onto_helm() {
    let err = p("helm:diff@url=https://github.com/databus23/helm-diff,allow_http").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("not an option on `helm`"), "{}", msg);
    assert!(
        !err.hint.unwrap().contains("helm"),
        "http's refusal must not name helm as a backend that takes it"
    );
}
