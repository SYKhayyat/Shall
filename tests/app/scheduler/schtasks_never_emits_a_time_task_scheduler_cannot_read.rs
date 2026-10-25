use super::*;

/// The property, not the cases: whatever a cron says, the time handed to Task Scheduler is
/// always `HH:mm`. This is the assertion that would have caught the reported defect without
/// anyone thinking of `0 3 * * *` in particular.
#[test]
fn schtasks_never_emits_a_time_task_scheduler_cannot_read() {
    for cron in [
        "0 3 * * *",
        "@daily",
        "@weekly",
        "@monthly",
        "@yearly",
        "@hourly",
        "5 9 * * *",
        "0 0 * * *",
        "59 23 * * *",
        "30 4 1 * *",
        "0 3 * * 1",
        "0 */6 * * *",
        "7 7 7 7 *",
    ] {
        let args = map_cron_to_schtasks(cron).unwrap_or_else(|e| panic!("{cron}: {e}"));
        if let Some(i) = args.iter().position(|a| a == "/ST") {
            let st = &args[i + 1];
            let (h, m) = st.split_once(':').unwrap_or_else(|| panic!("{cron}: {st}"));
            assert!(
                h.len() == 2 && m.len() == 2,
                "`{cron}` produced /ST {st}, which Task Scheduler rejects"
            );
            assert!(
                h.chars().chain(m.chars()).all(|c| c.is_ascii_digit()),
                "`{cron}` produced /ST {st}, which is not a time"
            );
            assert!(
                h.parse::<u8>().unwrap() < 24 && m.parse::<u8>().unwrap() < 60,
                "`{cron}` produced /ST {st}, which is not a real time"
            );
        }
    }
}
