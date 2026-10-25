use super::*;

/// Every option, against every scheduler, stated as a table — because a matrix checked one
/// cell at a time is a matrix with a hole in it. `Ok` means the scheduler expresses it;
/// `Err` means it says so by name rather than accepting the option and dropping it.
#[test]
fn every_option_is_either_expressed_or_refused_by_name() {
    let with = |f: fn(&mut ScheduleConfig)| {
        let mut c = cfg("0 2 * * *");
        f(&mut c);
        c
    };
    let boot = |f: fn(&mut ScheduleConfig)| {
        let mut c = cfg("@reboot");
        f(&mut c);
        c
    };
    let cases: &[(&str, ScheduleConfig, bool, bool, bool)] = &[
        // option            declaration                                 systemd  launchd  windows
        (
            "enabled=false",
            with(|c| c.enabled = Some(false)),
            true,
            true,
            true,
        ),
        (
            "enabled=true",
            with(|c| c.enabled = Some(true)),
            true,
            true,
            true,
        ),
        (
            "persistent=true",
            with(|c| c.persistent = Some(true)),
            true,
            true,
            false,
        ),
        (
            "persistent=false",
            with(|c| c.persistent = Some(false)),
            true,
            false,
            false,
        ),
        ("jitter", with(|c| c.jitter = Some(900)), true, false, false),
        (
            "elevated=true",
            with(|c| c.elevated = Some(true)),
            false,
            false,
            true,
        ),
        (
            "elevated=false",
            with(|c| c.elevated = Some(false)),
            true,
            true,
            true,
        ),
        // A timer option on a job that has no timer.
        (
            "jitter@reboot",
            boot(|c| c.jitter = Some(900)),
            false,
            false,
            false,
        ),
        (
            "persistent@reboot",
            boot(|c| c.persistent = Some(true)),
            false,
            false,
            false,
        ),
    ];
    for (what, config, systemd, launchd, windows) in cases {
        let got = (
            LinuxSystemdProvisioner.refuse_unsupported(config).is_ok(),
            MacLaunchdProvisioner.refuse_unsupported(config).is_ok(),
            WindowsTaskProvisioner.refuse_unsupported(config).is_ok(),
        );
        assert_eq!(
            got,
            (*systemd, *launchd, *windows),
            "{} is expressed/refused differently than the table says",
            what
        );
    }
}
