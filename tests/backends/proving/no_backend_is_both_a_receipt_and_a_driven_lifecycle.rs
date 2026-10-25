use super::*;

/// The two tables answer different questions, so a name may be in one and not the other — but
/// never in both, which would mean a receipt for a lifecycle a harness has run.
#[test]
fn no_backend_is_both_a_receipt_and_a_driven_lifecycle() {
    for (backend, _) in UNPROVEN {
        assert!(
            !DRIVEN.contains(backend),
            "`{backend}` is in `UNPROVEN` with a receipt saying no harness has driven it, and \
             in `DRIVEN` saying one has"
        );
    }
}
