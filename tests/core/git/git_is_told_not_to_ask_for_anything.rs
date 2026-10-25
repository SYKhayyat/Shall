use super::*;

/// **A closed stdin does not stop a credential prompt** — git reads it from `/dev/tty`, the
/// same way sudo does, which is what made `S88` a fifteen-minute silence rather than an
/// error. Asserted on the command Shall builds, because the failure it guards against has
/// no output to match on: the run simply never returns.
#[test]
fn git_is_told_not_to_ask_for_anything() {
    let mut cmd = std::process::Command::new("git");
    GitManager::ask_nothing(&mut cmd);
    let env: Vec<(String, Option<String>)> = cmd
        .get_envs()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.map(|v| v.to_string_lossy().into_owned()),
            )
        })
        .collect();
    assert!(
        env.contains(&("GIT_TERMINAL_PROMPT".to_string(), Some("0".to_string()))),
        "git will still prompt for a credential: {env:?}"
    );
    assert!(
        env.contains(&("GCM_INTERACTIVE".to_string(), Some("never".to_string()))),
        "Git Credential Manager will still open a window nobody sees: {env:?}"
    );
}
