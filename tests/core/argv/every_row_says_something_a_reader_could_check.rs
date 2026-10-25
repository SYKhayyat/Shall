use super::*;

/// An exemption is a claim, and a claim has content. Both kinds of evidence say something
/// about the parser; neither may be a shrug or a date.
#[test]
fn every_row_says_something_a_reader_could_check() {
    for t in TERMINATORS {
        let (kind, text) = match t.evidence {
            Evidence::Measured(w) => ("measured", w),
            Evidence::Unasked(w) => ("unasked", w),
            Evidence::Divergent(w) => ("divergent", w),
        };
        assert!(
            text.len() > 40,
            "`{}`'s {kind} evidence has no substance: {text:?}",
            t.binary
        );
        let lower = text.to_lowercase();
        assert!(
            !lower.contains("not yet") && !lower.contains("todo"),
            "`{}`'s evidence is a schedule, not a claim about its parser: {text:?}",
            t.binary
        );
    }
}
