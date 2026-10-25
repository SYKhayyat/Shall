use super::*;

/// A default on a table the parent does not use is a default nothing reads, and `[query]`
/// is a table whose whole content is a number whose absence means "ask once".
#[test]
fn the_query_table_defaults_to_asking_once() {
    assert_eq!(QuerySettings::default().retries, 0);
    assert_eq!(Config::default().query.retries, 0);
}
