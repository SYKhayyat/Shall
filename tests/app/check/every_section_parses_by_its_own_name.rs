use super::*;

#[test]
fn every_section_parses_by_its_own_name() {
    for s in Section::ALL {
        assert_eq!(Section::parse(s.as_str()), Some(s));
    }
    // Case and surrounding space are a typing detail, not a different word.
    assert_eq!(Section::parse("  Health "), Some(Section::Health));
}
