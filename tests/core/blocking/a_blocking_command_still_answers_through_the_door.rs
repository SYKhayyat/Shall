use super::*;

/// The third door works, and the answer comes back. Run under a multi-thread runtime,
/// because that is the flavour whose worker was being parked.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_blocking_command_still_answers_through_the_door() {
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "echo through-the-door"]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", "echo through-the-door"]);
        c
    };
    let out = command_output(&mut cmd).expect("the shell ran");
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("through-the-door"));
}
