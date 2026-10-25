use super::*;

/// The lists and the `match` arms are two spellings of one fact, and this is what keeps
/// them one. A name in the list that the match does not answer to is a row that would fail
/// at load; a match arm missing from the list is a reader the coverage gate cannot see.
#[test]
fn every_listed_reader_resolves() {
    for n in INSTALLED_NAMES {
        assert!(installed(n).is_some(), "INSTALLED_NAMES lists `{n}`");
    }
    for n in SEARCH_NAMES {
        assert!(search(n).is_some(), "SEARCH_NAMES lists `{n}`");
    }
    for n in PROBE_NAMES {
        assert!(probe(n).is_some(), "PROBE_NAMES lists `{n}`");
    }
    for n in NAMES_NAMES {
        assert!(names(n).is_some(), "NAMES_NAMES lists `{n}`");
    }
}
