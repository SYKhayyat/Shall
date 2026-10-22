use super::*;

/// A `Prefix` builds a statement or is parsed above the dispatch loop; a `Directive` or a
/// `Foreign` word never builds one. A `build` on a non-prefix would be dead code the
/// dispatch loop can never reach, because it only strips spellings ending in `:`.
#[test]
fn only_prefixes_build_statements() {
    for kw in KEYWORDS {
        if kw.build.is_some() {
            assert!(
                kw.takes_colon(),
                "`{}` builds a statement but is not a prefix",
                kw.word()
            );
        }
    }
}
