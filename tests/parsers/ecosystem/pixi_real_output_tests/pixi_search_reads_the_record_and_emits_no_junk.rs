use super::*;

/// The rule this enforces repo-wide: **a parser is tested against output captured from the
/// tool it parses, and from no other tool.** `names_only` serves five managers and its only
/// test used a spack fixture — it passed, and said nothing whatever about pixi, which is
/// exactly where it was wrong.
#[test]
fn pixi_search_reads_the_record_and_emits_no_junk() {
    let found = pixi_search(SEARCH, "pixi");
    assert_eq!(found.len(), 1, "one search, one package: {found:?}");
    assert_eq!(found[0].name, "ripgrep");
    assert_eq!(found[0].version.as_deref(), Some("15.2.0"));
}
