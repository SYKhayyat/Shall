use super::*;

#[test]
fn a_failing_shell_command_throws() {
    let cmd = if cfg!(windows) {
        "cmd /c exit 2"
    } else {
        "exit 2"
    };
    let err = run(&format!(r#"#{{ x: sh("{}") }}"#, cmd)).unwrap_err();
    assert!(err.what.contains("sh:"), "{}", err);
}
