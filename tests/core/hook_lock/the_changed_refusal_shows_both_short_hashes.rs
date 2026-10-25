use super::*;

#[test]
fn the_changed_refusal_shows_both_short_hashes() {
    let was = "a".repeat(64);
    let now = "b".repeat(64);
    let msg = refusal(
        "after_install:nginx",
        "local",
        &Verdict::Changed {
            was: was.clone(),
            now: now.clone(),
        },
    );
    assert!(msg.contains("was: sha256:aaaaaaaaaaaa"));
    assert!(msg.contains("now: sha256:bbbbbbbbbbbb"));
    // The full 64-char hash is not dumped into the message.
    assert!(!msg.contains(&was));
}
