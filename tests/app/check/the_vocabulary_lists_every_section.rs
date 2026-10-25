use super::*;

/// The vocabulary is what the error prints, so it must carry every section — otherwise a
/// user is told to pick from a list that is missing the one they wanted.
#[test]
fn the_vocabulary_lists_every_section() {
    let v = Section::vocabulary();
    for s in Section::ALL {
        assert!(v.contains(s.as_str()), "{} missing from {}", s, v);
    }
}
