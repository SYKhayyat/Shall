use super::*;

#[test]
fn unprotect_wins_over_the_os_essential_flag() {
    // The documented promise: un-protect beats *everything*, OS flags included.
    // Previously the OS check ran in an `else if` and fired anyway.
    let mut cfg = config_with(20);
    cfg.guard.unprotected_packages = vec!["dash".into()];
    let os: HashSet<String> = ["apt:dash".to_string()].into_iter().collect();
    assert!(protection_of(&cfg, Some("apt"), "dash", &os).is_none());
    // An essential package the user did NOT exempt is still protected.
    let os2: HashSet<String> = ["apt:base-files".to_string()].into_iter().collect();
    assert!(protection_of(&cfg, Some("apt"), "base-files", &os2).is_some());
}
