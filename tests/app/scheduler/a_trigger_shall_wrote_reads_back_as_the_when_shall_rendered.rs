use super::*;

#[test]
fn a_trigger_shall_wrote_reads_back_as_the_when_shall_rendered() {
    let cases: &[(&str, &str)] = &[
        (
            "<CalendarTrigger id=\"Trigger1\"><StartBoundary>2026-07-06T03:00:00\
             </StartBoundary><ScheduleByDay><DaysInterval>1</DaysInterval></ScheduleByDay>\
             </CalendarTrigger>",
            "daily at 03:00",
        ),
        (
            "<CalendarTrigger><StartBoundary>2010-12-16T09:00:00</StartBoundary>\
             <ScheduleByWeek><WeeksInterval>1</WeeksInterval><DaysOfWeek><Monday />\
             <Wednesday /></DaysOfWeek></ScheduleByWeek></CalendarTrigger>",
            "weekly on MON,WED at 09:00",
        ),
        (
            "<CalendarTrigger><StartBoundary>2010-12-16T04:30:00</StartBoundary>\
             <ScheduleByMonth><DaysOfMonth><Day>1</Day></DaysOfMonth><Months><January />\
             </Months></ScheduleByMonth></CalendarTrigger>",
            "monthly in JAN on 1 at 04:30",
        ),
        ("<BootTrigger><Delay>PT30S</Delay></BootTrigger>", "boot"),
        (
            "<TimeTrigger><StartBoundary>1992-05-01T00:00:00</StartBoundary>\
             <Repetition><Interval>PT15M</Interval></Repetition></TimeTrigger>",
            "every 15m",
        ),
        (
            "<TimeTrigger><StartBoundary>1992-05-01T00:00:00</StartBoundary>\
             <Repetition><Interval>PT6H</Interval></Repetition></TimeTrigger>",
            "every 6h from 00:00",
        ),
    ];
    for (triggers, want) in cases {
        let doc = task_xml(triggers, "true", "");
        assert_eq!(
            when_from_xml(&doc).as_deref(),
            Some(*want),
            "trigger read wrong: {}",
            triggers
        );
    }
}
