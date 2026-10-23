use super::*;

#[test]
fn ordinary_nonzero_is_never_benign() {
    for (policy, cmd, code) in [
        (exit_policy::apk(), "apk", 1),
        (exit_policy::apt(), "apt-get", 100),
        (exit_policy::dnf(), "dnf", 1),
    ] {
        assert!(executor_for(policy)
            .ensure_status(cmd, finished(code, "", "boom"))
            .is_err());
    }
}
