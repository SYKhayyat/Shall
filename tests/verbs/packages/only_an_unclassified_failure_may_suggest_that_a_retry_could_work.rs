use super::*;

/// Every reason a line can stay, and the sentence each one earns. Enumerated from the
/// enum rather than sampled on a host, because the wording half of E1 came back by
/// growing a fourth situation that the single `else` covering the other three still
/// answered with "`sync` will try it again".
#[test]
fn only_an_unclassified_failure_may_suggest_that_a_retry_could_work() {
    let file = std::path::Path::new("modules/imperative.txt");
    for why in [
        WhyKept::Refused,
        WhyKept::Exhausted,
        WhyKept::NameAbsentElsewhere,
        WhyKept::Unclassified,
    ] {
        let advice = kept_line_advice(why, "npm:cowsay", file);
        // Every branch, without exception: where the line is, and the way out of it.
        assert!(
            advice.contains("modules/imperative.txt"),
            "{why:?} does not name the file the line is in: {advice}"
        );
        assert!(
            advice.contains("shall unmanage npm:cowsay"),
            "{why:?} does not name the command that removes it: {advice}"
        );
        let promises_a_retry = advice.contains("`sync` will try it again");
        assert_eq!(
            promises_a_retry,
            why == WhyKept::Unclassified,
            "{why:?} earns the wrong sentence — only an unclassified failure may suggest \
             that trying again could work, because every other case has already been \
             shown otherwise: {advice}"
        );
    }
}
