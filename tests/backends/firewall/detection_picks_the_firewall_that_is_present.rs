use super::*;

#[test]
fn detection_picks_the_firewall_that_is_present() {
    let all = adapters(vec![]);
    let only_ufw = |cmd: &str| cmd == "ufw";
    let found = detect(&all, &only_ufw);
    // On a non-Linux host the ufw row does not apply, and that is the right answer.
    if cfg!(target_os = "linux") {
        assert_eq!(found.map(|a| a.name.as_str()), Some("ufw"));
    } else {
        assert!(found.is_none());
    }
    assert!(detect(&all, &|_| false).is_none());
}
