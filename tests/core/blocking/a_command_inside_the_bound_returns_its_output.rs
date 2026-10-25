use super::*;

/// The bound must not cost the ordinary case its output. A command inside the bound comes
/// back whole — both streams, and the exit status.
#[test]
fn a_command_inside_the_bound_returns_its_output() {
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.args(["/c", "echo hello"]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", "echo hello"]);
        c
    };
    let out = command_output_within(
        &mut cmd,
        "a probe",
        Some(std::time::Duration::from_secs(30)),
    )
    .expect("a fast command");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello");
}
