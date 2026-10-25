use super::*;

/// The other half of the same rule, asked of every backend rather than of winget.
///
/// `AllInstalled` asserts two things at once — the manager invents no dependencies **and**
/// it can reinstall everything it lists — and winget was filed under it because only the
/// first was ever checked. Whether each *other* manager on that list satisfies the second
/// is unverified and is the open sweep; this pins the one answer that is measured, so a
/// tidy-up cannot quietly put winget back.
#[tokio::test]
async fn winget_is_not_among_the_managers_that_adopt_their_whole_listing() {
    let reg = build_registry().await;
    let from_listing: Vec<String> = reg
        .present_on_this_machine()
        .iter()
        .filter(|b| {
            b.as_queryable()
                .is_some_and(|q| q.manual_source().starts_with("everything "))
        })
        .map(|b| b.name().to_string())
        .collect();
    assert!(
        !from_listing.iter().any(|n| n == "winget"),
        "winget adopts from its listing again (Q36): {from_listing:?}"
    );
}
