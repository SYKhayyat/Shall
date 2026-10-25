use super::*;

#[test]
fn the_timeout_message_names_the_token_and_the_identity() {
    let msg = token_timeout_message(
        Path::new("/cfg/api.age"),
        Path::new("/home/a/.age/yubikey.txt"),
        Some("yubikey"),
    );
    assert!(msg.contains("yubikey"), "{}", msg);
    assert!(msg.contains("/cfg/api.age"), "{}", msg);
    assert!(msg.contains("yubikey.txt"), "{}", msg);
    assert!(msg.contains("30s"), "{}", msg);
}
