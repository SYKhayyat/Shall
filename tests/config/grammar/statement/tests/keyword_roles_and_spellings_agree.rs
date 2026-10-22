use super::*;

/// The role is what a word is; the colon is how it is written. Nothing stops an entry
/// declaring one and spelling the other, and the dispatch loop strips `spelling` while
/// every quantifier asks `role` — so a mismatch would make a prefix invisible to the spec
/// ratchet while it kept on parsing.
#[test]
fn keyword_roles_and_spellings_agree() {
    for kw in KEYWORDS {
        assert_eq!(
            kw.takes_colon(),
            kw.spelling.ends_with(':'),
            "`{}` is spelled {:?} but its role is {:?}",
            kw.word(),
            kw.spelling,
            kw.role
        );
    }
}
