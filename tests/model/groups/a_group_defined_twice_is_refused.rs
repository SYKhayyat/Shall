use super::*;

/// A group defined twice is an error, not a silent last-wins: two definitions of one name is
/// a file where the reader cannot tell which one is in force.
#[test]
fn a_group_defined_twice_is_refused() {
    assert!(Groups::parse("tools = apt\ntools = dnf\n").is_err());
}
