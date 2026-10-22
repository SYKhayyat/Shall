use super::*;

/// **The form this grammar tells you to write must be a form it accepts.**
///
/// `means` is not a comment. When a reserved word turns up somewhere it cannot go, the
/// error hands the reader this string as the way to say it instead — so a wrong one sends
/// somebody to a second refusal, which is worse than the first because it looks like the
/// program contradicting itself.
///
/// `service:` advertised `@state=running` and the option is `@status`. Nothing read these
/// strings, so the grammar documented an option the same file rejects, and the only reason
/// it surfaced is that an example config copied the advice and the examples gate refused it.
///
/// Only the prefixes are checked, and only the ones written out in full: several entries
/// are deliberately prose (`end` is "`}` — blocks close with a brace") and several carry a
/// `…` where a value belongs. A placeholder is honest documentation and a bad test subject,
/// so it is skipped by name rather than parsed and hoped for.
#[test]
fn every_prefix_advertises_a_line_this_grammar_accepts() {
    let permissive = |_: &str| true;
    let mut checked = 0;
    for kw in KEYWORDS {
        if kw.role != KeywordRole::Prefix || kw.means.contains('…') {
            continue;
        }
        checked += 1;
        if let Err(e) = parse(&o(), kw.means, &permissive) {
            panic!(
                "`{}` tells the reader to write `{}`, and this grammar refuses it: {}",
                kw.spelling, kw.means, e.what
            );
        }
    }
    // A loop that matched nothing passes by finding nothing, and this one filters twice.
    assert!(
        checked >= 6,
        "only {checked} prefix example(s) were checked; the filter above has stopped matching the table and this test is asserting almost nothing"
    );
}
