use super::*;

/// The config is mounted READ-ONLY. Without this the rehearsal could write to the repo it
/// was asked to inspect — and `try`'s whole claim is that it touches nothing on the host.
#[test]
fn the_config_is_mounted_read_only() {
    let cmd = argv("docker", "shall-it-ubuntu", "/home/a/.config/shall");
    let mount = cmd
        .iter()
        .find(|a| a.contains(":/shall-config"))
        .expect("the config must be mounted");
    assert!(mount.ends_with(":ro"), "{}", mount);
}
