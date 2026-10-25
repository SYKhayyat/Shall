use super::*;

/// A code in the table that never reaches the executor running the command is the gap that
/// left sixteen registrars with no policy at all — so this drives a real command through
/// `install` and reads the outcome, rather than asserting on the table it just read.
///
/// One row, one exit code, two verbs: `start` declares it benign and `stop` does not, so a
/// pass here cannot come from the command, the code, or the platform.
#[tokio::test]
async fn the_benign_code_reaches_the_verb_that_declared_it_and_no_other() {
    #[cfg(windows)]
    let toml = r#"
[[init]]
name = "probe"
detect = "cmd"
start = [["cmd", "/C", "exit 7"]]
stop  = [["cmd", "/C", "exit 7"]]
start_benign_exits = [7]
"#;
    #[cfg(not(windows))]
    let toml = r#"
[[init]]
name = "probe"
detect = "sh"
start = [["sh", "-c", "exit 7"]]
stop  = [["sh", "-c", "exit 7"]]
start_benign_exits = [7]
"#;
    let file: InitProviderFile = toml::from_str(toml).unwrap();
    let core = Arc::new(ServiceBackendCore::with_providers(
        CommandExecutor::new(false, false),
        file.init,
    ));
    let inst = ServiceInstallable { core };
    let spec = |status: &str| PackageSpec {
        name: "irrelevant".to_string(),
        backend: "service".to_string(),
        options: [("status", status)].into_iter().collect(),
        requires: Vec::new(),
        present: true,
    };

    inst.install(&[spec("running")], false)
        .await
        .expect("7 is declared benign for start: already running is the declared state");
    let err = inst
        .install(&[spec("stopped")], false)
        .await
        .expect_err("stop never declared 7 benign, so it is an ordinary failure");
    assert!(err.to_string().contains('7'), "{}", err);
}
