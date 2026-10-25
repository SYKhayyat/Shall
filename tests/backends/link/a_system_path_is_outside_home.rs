use super::*;

#[test]
fn a_system_path_is_outside_home() {
    #[cfg(windows)]
    let system = r"C:\ProgramData\shall\x";
    #[cfg(not(windows))]
    let system = "/etc/cron.d/x";
    assert!(is_outside_home(&resolve_target(system).unwrap()));
}
