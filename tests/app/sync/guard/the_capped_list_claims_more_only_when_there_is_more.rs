use super::*;

/// The cap says "â€¦and N more" only when there are more.
///
/// `protected.len() > MAX_LISTED` reads `>=` just as happily, and every test of this message
/// used 25 objections against a cap of 10 â€” far enough over that both comparisons agree.
/// At exactly the cap they disagree, and the mutant's version prints "â€¦and 0 more
/// protected package(s)" under a list that already showed every one of them.
#[test]
fn the_capped_list_claims_more_only_when_there_is_more() {
    let protected = |n: usize| GuardReport {
        objections: (0..n)
            .map(|i| Objection::Protected {
                key: format!("apt:pkg{}", i),
                reason: "protected by config rule `x`".into(),
            })
            .collect(),
        ..Default::default()
    };

    let exactly = protected(MAX_LISTED).message(GuardScope::Sync, RemovalKind::Package);
    assert!(
        !exactly.contains("â€¦and"),
        "ten protected packages all fit under a cap of ten, so there is no remainder to \
             announce:\n{exactly}"
    );
    assert!(
        exactly.contains(&format!("apt:pkg{}", MAX_LISTED - 1)),
        "the last one that fits must still be listed:\n{exactly}"
    );

    let one_over = protected(MAX_LISTED + 1).message(GuardScope::Sync, RemovalKind::Package);
    assert!(
        one_over.contains("â€¦and 1 more protected package(s)"),
        "eleven against a cap of ten leaves exactly one unlisted:\n{one_over}"
    );
}
