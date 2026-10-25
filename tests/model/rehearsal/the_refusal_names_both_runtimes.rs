use super::*;

/// 7h: with no runtime it refuses and names what is missing, rather than running anywhere.
#[test]
fn the_refusal_names_both_runtimes() {
    let msg = no_runtime_refusal();
    assert!(msg.contains("docker"), "{}", msg);
    assert!(msg.contains("podman"), "{}", msg);
    assert!(msg.contains("refusing"), "{}", msg);
}
