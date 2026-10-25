use super::*;

/// A shape Shall does not write is **unreadable**, never drift. Reported as a mismatch it
/// would rewrite the task on every sync for ever and keep `check` red on something the
/// reader simply did not understand — V.188's rule, on a different store.
#[test]
fn a_trigger_shall_does_not_write_is_unreadable_rather_than_wrong() {
    for triggers in [
        // Somebody added a second trigger by hand.
        "<CalendarTrigger><StartBoundary>2026-07-06T03:00:00</StartBoundary><ScheduleByDay>\
         <DaysInterval>1</DaysInterval></ScheduleByDay></CalendarTrigger>\
         <LogonTrigger><Delay>PT5M</Delay></LogonTrigger>",
        // Every third day is not a schedule Shall can write.
        "<CalendarTrigger><StartBoundary>2026-07-06T03:00:00</StartBoundary><ScheduleByDay>\
         <DaysInterval>3</DaysInterval></ScheduleByDay></CalendarTrigger>",
        // An event trigger, which has no cron at all.
        "<EventTrigger><Subscription>x</Subscription></EventTrigger>",
        // A shape this reader understands, with a shape it does not standing beside it.
        // Counting only the kinds it knows would read the calendar trigger and report the
        // task as matching a declaration that says nothing about the event.
        "<CalendarTrigger><StartBoundary>2026-07-06T03:00:00</StartBoundary><ScheduleByDay>\
         <DaysInterval>1</DaysInterval></ScheduleByDay></CalendarTrigger>\
         <EventTrigger><Subscription>x</Subscription></EventTrigger>",
        // A day-scale repetition. Shall writes daily as a calendar trigger, never as a
        // repetition, so this is somebody else's task under Shall's name.
        "<TimeTrigger><StartBoundary>1992-05-01T01:00:00</StartBoundary>\
         <Repetition><Interval>P1D</Interval></Repetition></TimeTrigger>",
        // No triggers whatsoever.
        "",
    ] {
        let doc = task_xml(triggers, "true", "");
        assert_eq!(when_from_xml(&doc), None, "read as drift: {}", triggers);
    }
}
