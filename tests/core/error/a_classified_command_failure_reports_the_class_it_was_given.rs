use super::*;

/// The constructor exists to stop `unknown` being the only answer a summary can give.
#[test]
fn a_classified_command_failure_reports_the_class_it_was_given() {
    for class in [
        Retryability::Transient,
        Retryability::Permanent,
        Retryability::Exhausted,
        Retryability::Unknown,
    ] {
        let e = Error::command_failed_classified("2 operation(s) failed", class);
        assert_eq!(e.retryability(), class);
        // A summary is about operations, not about a name. Reporting it as an absent name
        // would withdraw a declaration over a rate limit.
        assert!(!e.says_a_name_is_absent(), "{class:?}");
    }
}
