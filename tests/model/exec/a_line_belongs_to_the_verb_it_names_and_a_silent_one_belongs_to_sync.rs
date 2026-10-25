use super::*;

/// Which verb claims which line, as a table (`H6`).
///
/// **The `None` row is the ruling.** A manifest that says nothing about verbs belongs to
/// `sync` and to `sync` alone, which is what makes this change invisible to every `exec:`
/// line that already exists. The alternative — `upgrade` running everything — needs no
/// option, no grammar and no test, and hands a verb that has never executed a user script
/// every script somebody approved for a different verb.
#[test]
fn a_line_belongs_to_the_verb_it_names_and_a_silent_one_belongs_to_sync() {
    for (on, sync, upgrade, what) in [
        (None, true, false, "silent: today's meaning, unchanged"),
        (Some("sync"), true, false, "written out"),
        (Some("upgrade"), false, true, "the step this was built for"),
        (Some("both"), true, true, "the third case"),
        (
            Some(" upgrade "),
            false,
            true,
            "trimmed like every other value",
        ),
        (
            Some("nonsense"),
            true,
            false,
            "unknown reads as the default, never as both",
        ),
    ] {
        assert_eq!(Verb::Sync.claims(on), sync, "sync/{on:?}: {what}");
        assert_eq!(Verb::Upgrade.claims(on), upgrade, "upgrade/{on:?}: {what}");
    }
}
