use super::*;

/// One binary, one row. The old shape was two tables and a test asserting they did not
/// contradict each other; a duplicate row is the same contradiction in one table.
#[test]
fn no_binary_is_listed_twice() {
    let mut seen: Vec<&str> = Vec::new();
    for t in TERMINATORS {
        assert!(
            !seen.contains(&t.binary),
            "`{}` has two rows — one invocation cannot both end its options and not",
            t.binary
        );
        seen.push(t.binary);
    }
}
