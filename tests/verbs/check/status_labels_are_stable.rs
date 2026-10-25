use super::*;

#[test]
fn status_labels_are_stable() {
    assert_eq!(status_label(HealthStatus::Ok), "OK");
    assert_eq!(status_label(HealthStatus::Degraded), "WARN");
    assert_eq!(status_label(HealthStatus::Critical), "FAIL");
    assert_eq!(status_label(HealthStatus::Absent), "absent");
}
