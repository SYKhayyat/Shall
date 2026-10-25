use super::*;

/// A harness canary is not evidence about a tool whose output Shall has never seen. Four rows
/// are canaried and still stamped, and each of them is the exact shape the audit found: a
/// claim of standing over a row that admits the opposite.
#[test]
fn a_driven_backend_whose_bytes_were_never_captured_reads_unproven() {
    for backend in ["spack", "krew", "asdf", "slackpkg"] {
        assert!(
            DRIVEN.contains(&backend),
            "`{backend}` is named here as harness-driven; if the harness changed, the \\
             assertion below is not about what it was"
        );
        assert_eq!(
            unproven_reason(backend),
            Some(BYTES_NEVER_CAPTURED),
            "`{backend}` is driven by a harness and its row still says UNVERIFIED, so it \\
             must not read proven"
        );
    }
}
