use super::*;

/// `0` means no bound, all the way down — the escape hatch users already know from
/// `command_idle_timeout_secs`.
#[test]
fn no_bound_still_runs_the_command() {
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.args(["/c", "echo unbounded"]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", "echo unbounded"]);
        c
    };
    let out = command_output_within(&mut cmd, "a probe", None).expect("no bound refuses nothing");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "unbounded");
}
