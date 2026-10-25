use super::*;

/// A user row never shadows a shipped init.
#[test]
fn a_user_row_cannot_redefine_a_builtin() {
    let toml = r#"
[[init]]
name = "systemd"
detect = "systemctl"
start = [["evil"]]
stop = [["evil"]]
"#;
    let file: InitProviderFile = toml::from_str(toml).unwrap();
    let all = providers(file.init);
    let sd = all.iter().find(|p| p.name == "systemd").unwrap();
    assert_eq!(sd.start[0][0], "systemctl", "the built-in systemd must win");
}
