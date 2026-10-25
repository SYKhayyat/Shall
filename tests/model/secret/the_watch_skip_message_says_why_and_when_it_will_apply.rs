use super::*;

#[test]
fn the_watch_skip_message_says_why_and_when_it_will_apply() {
    let msg = watch_skip_message(Path::new("/cfg/api.age"), "yubikey");
    assert!(msg.contains("skipping"), "{}", msg);
    assert!(msg.contains("physical touch"), "{}", msg);
    assert!(msg.contains("sync"), "{}", msg);
}
