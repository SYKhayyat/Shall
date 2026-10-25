use super::*;

/// The network half stays retryable: helm plugins come from a git host.
#[test]
fn helm_losing_the_network_is_worth_another_attempt() {
    assert_eq!(
        helm().retryability(&ExitPolicy::haystack(
            b"",
            b"Error: could not resolve host github.com"
        )),
        Retryability::Transient
    );
}
