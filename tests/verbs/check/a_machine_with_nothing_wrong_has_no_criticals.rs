use super::*;

/// The whole point of Q2: a manager nobody installed is not a fault, so it cannot be
/// counted as one. Twenty-three of these on an ordinary Windows box read as `23 critical`.
#[test]
fn a_machine_with_nothing_wrong_has_no_criticals() {
    let reports: Vec<_> = ["apt", "brew", "pacman", "dnf", "zypper"]
        .iter()
        .map(|n| (n.to_string(), rep(HealthStatus::Absent)))
        .chain(std::iter::once((
            "scoop".to_string(),
            rep(HealthStatus::Ok),
        )))
        .collect();
    let (ok, degraded, critical, absent) = doctor_tally(&reports);
    assert_eq!((ok, degraded, critical), (1, 0, 0));
    assert_eq!(absent, 5);
}
