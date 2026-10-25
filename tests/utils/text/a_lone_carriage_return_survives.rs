use super::*;

/// A lone `\r` survives, because winget's progress spinner uses it to overwrite a line and
/// the caller has to decide which of the overwritten states it wanted.
#[test]
fn a_lone_carriage_return_survives() {
    assert_eq!(sanitize("first\rsecond"), "first\rsecond");
}
