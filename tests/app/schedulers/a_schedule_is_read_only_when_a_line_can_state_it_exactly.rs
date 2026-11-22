//! C41's reader, and its refusals. Every case here is a way a schedule could be *widened* into
//! the nearest thing a `schedule:` line can say — a boot job into a daily one, a stepped
//! `OnCalendar` into a plain hour — which reports success and fires at the wrong times.

use shall::app::schedulers::{
    calendar_of, crontab, name_from, on_calendar_to_cron, timers_in_listing, Calendar,
};

#[test]
fn a_system_crontab_carries_its_user_column_and_a_user_crontab_does_not() {
    // The same line means two different jobs in the two formats, and reading either as the other
    // is the classic cron bug: the account becomes the first word of the command.
    let system = crontab(
        "0 3 * * * root /usr/bin/backup --all\n",
        "etc",
        true,
        "cron-etc",
    );
    assert_eq!(system.found.len(), 1);
    assert_eq!(system.found[0].user.as_deref(), Some("root"));
    assert_eq!(system.found[0].command, "/usr/bin/backup --all");
    assert_eq!(system.found[0].cron, "0 3 * * *");

    let user = crontab(
        "0 3 * * * /usr/bin/backup --all\n",
        "me",
        false,
        "cron-user",
    );
    assert_eq!(user.found[0].user, None);
    assert_eq!(
        user.found[0].command, "/usr/bin/backup --all",
        "without a user column the whole tail is the command"
    );
}

#[test]
fn assignments_and_comments_are_not_jobs_and_a_boot_job_is_refused_by_name() {
    let text = "SHELL=/bin/sh\n\
                PATH = /usr/local/bin:/usr/bin\n\
                # a comment\n\
                \n\
                @reboot root /usr/bin/warm-cache\n\
                @daily root /usr/bin/rotate\n";

    let reading = crontab(text, "/etc/crontab", true, "cron-etc");
    assert_eq!(
        reading.found.len(),
        1,
        "one job: the daily one. {:?}",
        reading.found
    );
    assert_eq!(reading.found[0].cron, "@daily");

    let (name, reason) = &reading.refused[0];
    assert!(name.starts_with("cron-etc-"), "{}", name);
    assert!(
        reason.contains("@reboot") && reason.contains("calendar"),
        "the refusal names the field and the value: {}",
        reason
    );
}

#[test]
fn a_line_with_no_command_is_refused_rather_than_declared_empty() {
    let reading = crontab("0 3 * * *\n", "etc", false, "cron-user");
    assert!(reading.found.is_empty());
    assert_eq!(reading.refused.len(), 1);
    assert!(
        reading.refused[0].1.contains("no command"),
        "{}",
        reading.refused[0].1
    );
}

#[test]
fn the_calendars_a_cron_line_states_exactly_are_mapped_and_the_rest_are_not() {
    assert_eq!(on_calendar_to_cron("hourly").as_deref(), Some("0 * * * *"));
    assert_eq!(on_calendar_to_cron("daily").as_deref(), Some("0 0 * * *"));
    assert_eq!(on_calendar_to_cron("weekly").as_deref(), Some("0 0 * * 1"));
    assert_eq!(
        on_calendar_to_cron("*-*-* 03:30:00").as_deref(),
        Some("30 3 * * *")
    );
    assert_eq!(
        on_calendar_to_cron("Mon..Fri 09:00").as_deref(),
        Some("0 9 * * 1-5")
    );
    assert_eq!(
        on_calendar_to_cron("Sat,Sun 08:15:00").as_deref(),
        Some("15 8 * * 6,7"),
        "Sunday is 7 here: this tree's cron parser refuses 0 in that field, so a `0` produced a \
         schedules file that did not parse"
    );
    // Two shapes this host's own timers are written in, both of which the first version of this
    // mapping refused: `*` is a value in both dialects, and a weekday may precede the date part.
    assert_eq!(
        on_calendar_to_cron("*-*-* *:00:00").as_deref(),
        Some("0 * * * *"),
        "`logrotate.timer` is `hourly`, and an asterisk hour is still an hour"
    );
    assert_eq!(
        on_calendar_to_cron("Mon *-*-* 00:00:00").as_deref(),
        Some("0 0 * * 1"),
        "`fstrim.timer` puts the weekday before the date"
    );

    // Refused, each for its own reason: a step is not one cron field, a seconds field that is
    // not `00` would fire at an unstated minute, two times of day are two jobs, and a floating
    // day-of-month has no cron spelling at all.
    for spec in [
        "*:0/15",
        "*-*-* 09:30:15",
        "09:00,17:00",
        "*-*-* 04:00:00~1",
        "*-*-03 04:00:00",
        "Mon 09:00, Fri 17:00",
    ] {
        assert_eq!(
            on_calendar_to_cron(spec),
            None,
            "`{}` must be refused, not rounded",
            spec
        );
    }
}

#[test]
fn an_unreadable_calendar_is_not_an_interval_and_neither_is_an_empty_schedule() {
    // The plain property, where the systemd in front of us prints it.
    assert_eq!(
        calendar_of("OnCalendar=*-*-* 02:00:00\n"),
        Calendar::Spec("*-*-* 02:00:00".to_string())
    );
    // And the structured form a newer systemd prints *instead* — this host's `show -p OnCalendar`
    // answers with nothing at all, so asking only for that property made every timer look like an
    // interval, including `OnCalendar=hourly` ones.
    assert_eq!(
        calendar_of("TimersCalendar={ OnCalendar=*-*-* *:00:00 ; next_elapse=Sun 08:00:00 EST }\n"),
        Calendar::Spec("*-*-* *:00:00".to_string())
    );
    // A real interval names its field, so the refusal can quote it — in the microsecond
    // spelling `TimersMonotonic` actually prints, reported under the unit file's own name.
    assert_eq!(
        calendar_of("TimersMonotonic={ OnBootUSec=15min ; next_elapse=15min }\n"),
        Calendar::Interval("OnBootSec".to_string())
    );
    assert_eq!(
        calendar_of("OnBootSec=15min\n"),
        Calendar::Interval("OnBootSec".to_string())
    );
    // Nothing printed is *unreadable*, not "no calendar": the two are different findings and
    // reporting the second as the first tells the person their daily timer is an interval.
    assert_eq!(calendar_of(""), Calendar::Unreadable);
    assert_eq!(calendar_of("OnCalendar=\n"), Calendar::Unreadable);
}

#[test]
fn the_unit_is_found_even_when_the_timer_has_never_fired() {
    // `n/a` in the first four columns is the fresh-machine case, and it is most timers.
    let listing = "NEXT                        LEFT     LAST  PASSED  UNIT                  ACTIVATES\n\
                   n/a                         n/a      n/a   n/a     shall-nightly.timer   shall-nightly.service\n\
                   Wed 2026-11-25 03:00:00 UTC 2 days   n/a   n/a     logrotate.timer       logrotate.service\n";
    assert_eq!(
        timers_in_listing(listing),
        vec!["shall-nightly.timer", "logrotate.timer"]
    );
}

#[test]
fn a_name_keeps_the_dialect_and_squashes_only_the_characters_a_line_cannot_carry() {
    assert_eq!(
        name_from("timer-system", "my-timer.timer"),
        "timer-system-my-timer-timer"
    );
    assert_eq!(
        name_from("cron", "etc-cron-d/log-rotate"),
        "cron-etc-cron-d-log-rotate"
    );
}
