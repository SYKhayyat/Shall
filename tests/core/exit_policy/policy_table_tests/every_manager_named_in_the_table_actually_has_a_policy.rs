use super::*;

/// **The list and the match must not drift.** `MANAGERS_WITH_A_POLICY` is what the auditing
/// gates walk, so a name that lost its policy would leave them auditing a bare default and
/// reporting nothing — the shape of failure this whole file's gates exist to refuse.
#[test]
fn every_manager_named_in_the_table_actually_has_a_policy() {
    let bare = ExitPolicy::default();
    for name in MANAGERS_WITH_A_POLICY {
        let p = for_manager(name);
        assert!(
            p.reads_output() || !p.benign_exits.is_empty() || !p.transient_exits.is_empty(),
            "`{name}` is in MANAGERS_WITH_A_POLICY and `for_manager` hands back a policy \
             that answers nothing — either the match arm went, or the name should"
        );
        assert!(
            !(p.benign_exits == bare.benign_exits
                && p.transient_exits == bare.transient_exits
                && !p.reads_output()),
            "`{name}` resolves to the default policy"
        );
    }
}
