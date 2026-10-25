use super::*;

/// systemd installed is not systemd running, and every Debian-family container proves it.
///
/// `systemctl` is on `PATH` in all of them and systemd is PID 1 in none, so detection picked
/// this row and `service:` failed with *"System has not been booted with systemd as init
/// system (PID 1). Can't operate. Failed to connect to bus: Host is down"* — on a host that
/// had `service` and `update-rc.d` sitting right beside it. Measured on the ubuntu image,
/// 2026-08-14, as three failures in section 14c.
///
/// Both directions are asserted, because the fix is only right if it is invisible on a real
/// systemd machine: with `/run/systemd/system` there, systemd still wins.
#[test]
fn systemd_installed_but_not_booted_loses_to_the_init_that_is_running() {
    use shall::core::adapter::first_present_on;
    let rows = providers(vec![]);
    // Both clients on PATH, which is exactly what a Debian container has.
    let both_installed = |c: &str| matches!(c, "systemctl" | "service");

    let in_a_container = first_present_on(&rows, "linux", &both_installed, &|_| false);
    assert_eq!(
        in_a_container.map(|r| r.name.as_str()),
        Some("sysvinit"),
        "systemd was chosen on a machine where it is installed and not running"
    );

    let on_a_real_box = first_present_on(&rows, "linux", &both_installed, &|p| {
        p == "/run/systemd/system"
    });
    assert_eq!(
        on_a_real_box.map(|r| r.name.as_str()),
        Some("systemd"),
        "the liveness check changed the answer on a machine that HAS booted systemd"
    );
}
