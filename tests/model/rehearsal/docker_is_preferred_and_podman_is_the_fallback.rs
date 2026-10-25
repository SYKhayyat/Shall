use super::*;

#[test]
fn docker_is_preferred_and_podman_is_the_fallback() {
    assert_eq!(pick_runtime(&|c| c == "docker"), Some("docker"));
    assert_eq!(pick_runtime(&|c| c == "podman"), Some("podman"));
    assert_eq!(pick_runtime(&|_| true), Some("docker"));
    assert_eq!(pick_runtime(&|_| false), None);
}
