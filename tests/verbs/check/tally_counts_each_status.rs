use super::*;

#[test]
fn tally_counts_each_status() {
    let reports = vec![
        ("apt".to_string(), rep(HealthStatus::Ok)),
        ("brew".to_string(), rep(HealthStatus::Ok)),
        ("snap".to_string(), rep(HealthStatus::Degraded)),
        ("nix".to_string(), rep(HealthStatus::Critical)),
        ("pacman".to_string(), rep(HealthStatus::Absent)),
        ("dnf".to_string(), rep(HealthStatus::Absent)),
    ];
    assert_eq!(doctor_tally(&reports), (2, 1, 1, 2));
}
