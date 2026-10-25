use super::*;

/// A cached listing may inform a report; it may never source a decision that outlives the
/// run. Asserted by name because the cost of getting it wrong is silent: a plan built on a
/// listing taken before the user removed something skips the install and reports success.
///
/// The allowlist is checked in both directions — every reader can be served, and every
/// command that writes its answer down cannot — so a verb added to one list and not the
/// other fails here rather than in somebody's manifest.
#[test]
fn only_a_command_that_just_reports_may_be_answered_from_disk() {
    for reader in ["list", "search", "check", "outdated", "info", "why"] {
        assert!(
            InstalledListings::cache_may_answer(reader),
            "`{reader}` only reports, and pays for a listing it could have had"
        );
    }
    for writer in [
        "sync",
        "install",
        "uninstall",
        "upgrade",
        "adopt",
        "plan",
        "heal",
        "rollback",
        "purge-undeclared",
    ] {
        assert!(
            !InstalledListings::cache_may_answer(writer),
            "`{writer}` writes down what it was told, and was told by a cache"
        );
    }
}
