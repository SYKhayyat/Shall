use super::*;

/// The other four shipped inits report "already in that state" as exit 0, so they declare
/// nothing here. Pinned rather than left blank: a row that grows a code silently, or loses
/// one, is the same invisible change either way.
#[test]
fn the_inits_that_answer_zero_declare_no_benign_codes() {
    for name in ["systemd", "openrc", "sysvinit", "launchd"] {
        let p = shipped(name);
        assert!(
            p.start_benign_exits.is_empty() && p.stop_benign_exits.is_empty(),
            "{} declares benign exits — if that is real it needs a measurement beside it",
            name
        );
    }
}
