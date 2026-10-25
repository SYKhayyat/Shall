use super::*;

#[test]
fn the_new_refusal_names_the_hook_and_the_fix() {
    let msg = refusal("after_install:nginx", "github:acme/fonts", &Verdict::New);
    assert!(msg.contains("after_install:nginx"));
    assert!(msg.contains("github:acme/fonts"));
    assert!(msg.contains("shall lock"));
}
