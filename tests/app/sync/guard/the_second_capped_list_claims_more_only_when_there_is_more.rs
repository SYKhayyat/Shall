use super::*;

/// The second capped list claims a remainder only when there is one.
///
/// **The same cap, asked twice, and only one of the two was pinned.** The guard renders two
/// capped lists: the packages a rule protects, and the packages behind a manager that could
/// not answer which ones the OS needs. The first was pinned at its boundary by
/// `the_capped_list_claims_more_only_when_there_is_more`; **the second was not pinned at all**,
/// and the mutation gate said so twice: the comparison in `GuardReport::message` that decides
/// whether the second list has a remainder, mutated to `==` and to `>=`, both survived the
/// whole suite.
///
/// **What each mutant does.** `> → ==` with eleven unverified objections never prints the
/// "…and N more" line at all — ten are listed, the eleventh is dropped without a word, and
/// the message never mentions it. The **refusal** is still correct (the guard's decision never
/// reads that line), but the *explanation* under-reports what Shall will not remove. `> → >=`
/// at exactly ten prints "…and 0 more through managers that could not answer" — a remainder
/// announced as zero.
///
/// **And the third comparison at the same site, `> → <`, is caught only by an arithmetic
/// underflow**: `unverified.len() - MAX_LISTED` panics in a debug build, and **wraps in a
/// release build** — where the mutation gate does not run, and where the release scripts do.
/// So the mutant the debug gate rejects would print "…and 18446744073709551615 more" in a
/// shipped build. **The property assertion below is the one that survives that**, and it is
/// why the test is a property and not a pair of boundary checks.
#[test]
fn the_second_capped_list_claims_more_only_when_there_is_more() {
    let unverified = |n: usize| GuardReport {
        objections: (0..n)
            .map(|i| Objection::UnverifiedEssentials {
                key: format!("apt:pkg{}", i),
                backend: "apt".to_string(),
            })
            .collect(),
        ..Default::default()
    };

    // **Exactly at the cap there is no remainder**, so the message must not claim one: the
    // `> → ==` mutant prints "…and 0 more", and the `> → >=` mutant prints the same.
    let exactly = unverified(MAX_LISTED).message(GuardScope::Sync, RemovalKind::Package);
    assert!(
        !exactly.contains("â€¦and"),
        "ten packages that could not be checked all fit under a cap of ten, so there is no \
         remainder to announce:\n{exactly}"
    );
    assert!(
        exactly.contains(&format!("apt:pkg{}", MAX_LISTED - 1)),
        "the last one that fits must still be listed:\n{exactly}"
    );

    // **One over, and the remainder is named** — with the number, because "and some more" is
    // not a count a reader can check.
    let one_over = unverified(MAX_LISTED + 1).message(GuardScope::Sync, RemovalKind::Package);
    assert!(
        one_over.contains("â€¦and 1 more through managers that could not answer"),
        "eleven against a cap of ten leaves exactly one unlisted, and the message must say so:\n\
         {one_over}"
    );

    // **And the number announced is never larger than the number of objections** — the
    // property that holds in a release build, where the subtraction underflows instead of
    // panicking. This is the assertion the debug build's panic would otherwise have made
    // for us.
    for n in [MAX_LISTED, MAX_LISTED + 1, MAX_LISTED + 7, 40] {
        let message = unverified(n).message(GuardScope::Sync, RemovalKind::Package);
        if let Some(rest) = message.split("â€¦and ").nth(1) {
            let announced: usize = rest
                .split_whitespace()
                .next()
                .and_then(|w| w.parse().ok())
                .unwrap_or_else(|| panic!("the remainder is not a number: {message}"));
            assert!(
                announced <= n,
                "the message announces {announced} more objections out of {n}, which is more \
                 objections than there are:\n{message}"
            );
        }
    }
}
