use super::*;

/// The two ends of U39's one table. A source key the grammar has never heard of is
/// rejected as a misspelling, and the backend then refuses every line that carries it —
/// which is how the fix shipped the first time, and it took a real helm to notice.
#[test]
fn every_install_source_key_is_a_legal_option_key() {
    for (backend, key) in INSTALLS_FROM_SOURCE {
        assert!(
            shall::config::grammar::statement::package_option_keys().contains(key),
            "`@{}` is {}'s install source and the grammar would refuse it",
            key,
            backend
        );
        assert_eq!(install_source_key(backend), Some(*key));
        assert!(is_source_key(key));
    }
}
