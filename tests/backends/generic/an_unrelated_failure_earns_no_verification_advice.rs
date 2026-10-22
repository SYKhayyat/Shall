use super::*;

/// And the limit on both: a failure that is not about verification is not answered with
/// advice about signatures.
#[test]
fn an_unrelated_failure_earns_no_verification_advice() {
    for withheld in [false, true] {
        assert_eq!(
            verification_note(
                "helm",
                "helm",
                "--verify=false",
                false,
                withheld,
                "Error: could not resolve host github.com"
            ),
            None,
            "a network failure was answered with advice about signatures (withheld={withheld})"
        );
    }
}
