use super::*;

/// **An unanchored pattern matches at any depth, the way `.gitignore` reads**, and an
/// anchored one does not. `*.bak` is the basename at any level; `/*.bak` is the root's. That
/// is the one rule in this file a user is most likely to hold the other way round, so it is
/// asserted rather than described.
#[test]
fn a_pattern_is_anchored_only_when_it_says_so() {
    let anchored = Pattern::parse("/docs/").expect("a pattern");
    assert!(anchored.matches("docs", true));
    assert!(!anchored.matches("notes/docs", true));

    let anywhere = Pattern::parse("docs/").expect("a pattern");
    assert!(anywhere.matches("docs", true));
    assert!(anywhere.matches("notes/docs", true));

    // An unanchored name matches at any depth; the same pattern anchored to the root does not.
    let anywhere_bak = Pattern::parse("*.bak").expect("a pattern");
    assert!(anywhere_bak.matches("a.bak", false));
    assert!(anywhere_bak.matches("deep/nested/a.bak", false));
    let root_bak = Pattern::parse("/*.bak").expect("a pattern");
    assert!(root_bak.matches("a.bak", false));
    assert!(!root_bak.matches("deep/a.bak", false));

    // `*` stays inside one segment and `**` does not, which only shows where a pattern has a
    // slash in it: `a/**/b` is a path, so the wildcard has to cross directories.
    let middle = Pattern::parse("a/**/b").expect("a pattern");
    assert!(middle.matches("a/b", true), "`**` absorbs zero segments");
    assert!(middle.matches("a/x/b", true));
    assert!(middle.matches("a/x/y/b", true));
    assert!(
        !middle.matches("z/a/b", true),
        "a pattern naming a path is about that path, not about any path ending in it"
    );
}
