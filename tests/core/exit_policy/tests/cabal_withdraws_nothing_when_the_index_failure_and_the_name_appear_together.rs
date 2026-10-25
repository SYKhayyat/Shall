use super::*;

/// A machine whose index never arrived says both things at once on some verbs. The
/// transient guard in `names_an_absent_package` is what keeps the first from being read
/// as the second, and this is the case that guard exists for.
#[test]
fn cabal_withdraws_nothing_when_the_index_failure_and_the_name_appear_together() {
    assert!(!cabal().names_an_absent_package(&ExitPolicy::haystack(
        b"<repo>/root.json does not have enough signatures signed with the appropriate keys
Unknown package \"hello\".",
        b""
    )));
}
