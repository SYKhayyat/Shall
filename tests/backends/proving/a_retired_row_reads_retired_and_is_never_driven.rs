use super::*;

/// **A `RETIRED` row reads as retired, and a retired manager is never driven.**
///
/// This is the whole of `PLAN.md` #38: `eopkg` carried three negatives at once and read as
/// *unverified*, which is a claim about somebody's attention rather than about the project. It
/// has no publishable image on any public registry and no maintainer, so the bytes cannot be
/// captured by anyone and no run of this repository's harness changes that.
///
/// Asserted in both directions, because they fail differently. The first is the stamp; the
/// second is the check the stamp exists to make possible — **a retired row must never acquire a
/// harness lifecycle**, because a harness driving a dead project is either a canary nobody can
/// satisfy or a lie about coverage, and the roster is the one place that would say so.
#[test]
fn a_retired_row_reads_retired_and_is_never_driven() {
    assert!(
        retired_rows().contains("eopkg"),
        "the row still says UNVERIFIED, so its retirement is being read as somebody's \
         oversight rather than as a fact about Solus"
    );
    assert_eq!(
        unproven_reason("eopkg"),
        Some(RETIRED_UPSTREAM),
        "a retired manager reported as 'no harness has run it' reads as a coverage gap, which \
         is work nobody can do"
    );
    assert!(
        !is_proven("eopkg"),
        "a row whose bytes nobody can capture must never read proven"
    );
    assert!(
        !DRIVEN.contains(&"eopkg"),
        "the harness grew a lifecycle for a retired manager: either a canary nobody can \
         satisfy or a claim of coverage that does not exist"
    );
    // And the stamp is not `UNVERIFIED` wearing a new label — the two are different answers
    // and a row must not be able to be both.
    let eopkg = shall::backends::onboarder::builtin_rows()
        .into_iter()
        .find(|r| r.name == "eopkg")
        .and_then(|r| r.fixture)
        .expect("the eopkg row carries a fixture");
    assert!(eopkg.is_retired(), "the stamp is not RETIRED");
    assert!(
        !eopkg.is_unverified(),
        "a retired row must not also claim nobody looked, or the two states are one again"
    );
    assert!(
        !eopkg.is_verified(),
        "no byte of eopkg's output came from eopkg, whatever the stamp says"
    );
}
