use super::*;

/// A statement where the question does not arise does not take the key: a key that means
/// nothing where it is written is a key that gets written there and silently ignored.
#[test]
fn scope_is_refused_where_it_means_nothing() {
    for line in [
        "service:nginx@scope=system",
        "schedule:nightly@cron=@daily,run=sync,scope=user",
    ] {
        let err = pv(line).unwrap_err();
        assert!(err.what.contains("not an option"), "{}: {}", line, err);
    }
}
