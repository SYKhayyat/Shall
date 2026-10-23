use super::*;

#[test]
fn value_flags_are_exactly_what_clap_says_take_a_value() {
    let flags = global_value_flags();
    assert!(flags.contains("--config") && flags.contains("-c"));
    // Every bool global: named here, they would each eat the following token.
    for valueless in ["--progress", "--dry-run", "-y", "--yes", "-v", "-q"] {
        assert!(
            !flags.contains(valueless),
            "{} takes no value; listing it skips a real token",
            valueless
        );
    }
    // Deleted flags cannot linger: the list is derived, not maintained.
    for gone in ["-g", "--groups-dir", "-b", "--backend", "--no-global"] {
        assert!(!flags.contains(gone), "{} was deleted", gone);
    }
}
