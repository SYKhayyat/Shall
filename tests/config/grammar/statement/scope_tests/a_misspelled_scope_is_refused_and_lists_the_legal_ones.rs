use super::*;

/// A misspelling must not read as "the default" — that would be a line that looks like a
/// decision and behaves as if nobody made one.
#[test]
fn a_misspelled_scope_is_refused_and_lists_the_legal_ones() {
    for bad in [
        "shim:rg@scope=machine",
        "shim:rg@scope=global",
        "shim:rg@scope=User",
    ] {
        let err = pv(bad).unwrap_err();
        assert!(err.what.contains("invalid `scope="), "{}: {}", bad, err);
        let full = err.to_string();
        assert!(full.contains("user") && full.contains("system"), "{}", full);
    }
}
