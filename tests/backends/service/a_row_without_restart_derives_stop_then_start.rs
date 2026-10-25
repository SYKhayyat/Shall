use super::*;

/// A user row with no native restart still restarts — stop then start, derived from the two
/// required actions, so a niche init need not spell restart out.
#[test]
fn a_row_without_restart_derives_stop_then_start() {
    let toml = r#"
[[init]]
name = "dinit"
detect = "dinitctl"
enable = [["dinitctl", "enable", "{name}"]]
disable = [["dinitctl", "disable", "{name}"]]
start = [["dinitctl", "start", "{name}"]]
stop = [["dinitctl", "stop", "{name}"]]
"#;
    let file: InitProviderFile = toml::from_str(toml).unwrap();
    let all = providers(file.init);
    let dinit = all.iter().find(|p| p.name == "dinit").expect("dinit loads");
    let cmds = dinit.plan(ServiceAction::Restart, "web");
    assert_eq!(
        cmds,
        vec![
            (
                ServiceAction::Stop,
                vec!["dinitctl".to_string(), "stop".into(), "web".into()]
            ),
            (
                ServiceAction::Start,
                vec!["dinitctl".to_string(), "start".into(), "web".into()]
            ),
        ]
    );
}
