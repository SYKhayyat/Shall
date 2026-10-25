use super::*;

#[test]
fn the_template_documents_no_setting_that_would_disarm_the_guard() {
    // Three of these used to be real, and each was a way to make a routine sync delete
    // something: `[guard.enforce_on]` switched the guard off per command,
    // `prune_scope = "system"` made sync remove software it never installed, and
    // `prune_on_sync` decided whether sync was sync at all. A config file is copied
    // between machines and pasted from the internet — it must not be able to say any
    // of this (V.21, V.34, II.17).
    for gone in [
        "enforce_on",
        "prune_on_sync",
        "prune_scope",
        "protect_imperative",
    ] {
        assert!(
            !CONFIG_TEMPLATE.contains(gone),
            "`{}` is deleted, but the template still offers it",
            gone
        );
    }
}
