use super::*;

/// A scope nobody can parse is not a default to fall back on.
#[test]
fn a_scope_that_is_neither_word_is_refused() {
    for bad in ["User", "SYSTEM", "machine", "global", "true", ""] {
        let err = FlatpakBackendCore::scope_from_settings(Some(&settings(&[("scope", bad)])));
        assert!(err.is_err(), "`scope = \"{bad}\"` was accepted");
    }
}
