use super::*;

#[test]
fn a_derived_restart_derives_from_the_domain_it_is_in() {
    let toml = r#"
[[init]]
name = "dinit"
detect = "dinitctl"
enable = [["dinitctl", "enable", "{name}"]]
disable = [["dinitctl", "disable", "{name}"]]
start = [["dinitctl", "start", "{name}"]]
stop = [["dinitctl", "stop", "{name}"]]
user_start = [["dinitctl", "--user", "start", "{name}"]]
user_stop = [["dinitctl", "--user", "stop", "{name}"]]
"#;
    let file: InitProviderFile = toml::from_str(toml).unwrap();
    let dinit = providers(file.init)
        .into_iter()
        .find(|p| p.name == "dinit")
        .expect("dinit loads");
    let system = dinit.plan(ServiceAction::Restart, "web");
    let user = dinit.plan_in(ServiceAction::Restart, "web", Domain::User);
    assert!(system.iter().all(|(_, c)| !c.iter().any(|a| a == "--user")));
    assert!(user.iter().all(|(_, c)| c.iter().any(|a| a == "--user")));
}
