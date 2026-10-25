use super::*;

#[test]
fn the_runtime_and_image_land_in_the_command() {
    let cmd = argv("podman", "shall-it-alpine", "/cfg");
    assert_eq!(cmd[0], "podman");
    assert!(cmd.contains(&"shall-it-alpine".to_string()));
}
