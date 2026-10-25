use super::*;

#[test]
fn systemd_oncalendar_mapping() {
    let p = LinuxSystemdProvisioner;
    // every Monday 04:30 -> zero-padded time, full date wildcards, weekday name
    assert_eq!(p.map_cron_to_systemd("30 4 * * 1"), "Mon *-*-* 04:30:00");
    // daily midnight: no weekday constraint
    assert_eq!(p.map_cron_to_systemd("0 0 * * *"), "*-*-* 00:00:00");
    // @-macros pass through
    assert_eq!(p.map_cron_to_systemd("@daily"), "daily");
}
