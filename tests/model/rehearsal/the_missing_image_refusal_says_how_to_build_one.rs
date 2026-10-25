use super::*;

#[test]
fn the_missing_image_refusal_says_how_to_build_one() {
    let msg = missing_image_refusal("docker", "shall-it-ubuntu");
    assert!(msg.contains("docker build"), "{}", msg);
    assert!(
        msg.contains("docker/integration/Dockerfile.ubuntu"),
        "{}",
        msg
    );
    for (name, _) in IMAGES {
        assert!(msg.contains(name), "{} missing from {}", name, msg);
    }
}
