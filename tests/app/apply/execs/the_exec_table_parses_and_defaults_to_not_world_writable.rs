use super::*;

#[test]
fn the_exec_table_parses_and_defaults_to_not_world_writable() {
    let c: shall::config::Config = toml::from_str("[exec]\ntrust = \"owner-only\"\n").unwrap();
    assert_eq!(c.exec.trust, ExecTrust::OwnerOnly);
    let c: shall::config::Config = toml::from_str("").unwrap();
    assert_eq!(c.exec.trust, ExecTrust::NotWorldWritable);
    let err = toml::from_str::<shall::config::Config>("[exec]\ntrust = \"yolo\"\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("yolo"), "{err}");
}
