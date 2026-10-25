use super::*;

/// The generated sandbox actually names the command it is confining and honours the
/// network verdict — the two facts a user reading the `.wsb` would check first.
#[cfg(target_os = "windows")]
#[test]
fn the_wsb_names_the_command_and_honours_the_network_setting() {
    let cfg = SandboxConfig {
        allow_network: false,
        ..Default::default()
    };
    let wsb = Sandbox::generate_wsb_config("cargo", &["build".to_string()], &cfg);
    assert!(wsb.contains("<Command>cargo build</Command>"), "{wsb}");
    assert!(wsb.contains("<Networking>Disable</Networking>"), "{wsb}");
}
