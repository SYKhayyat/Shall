use super::*;

/// A cron that names its weekdays out of order is the same schedule as one that names them
/// in order — `<DaysOfWeek>` is a set and comes back Sunday-first, so the declaration is
/// sorted the same way or every such schedule reports drift for ever.
#[test]
fn weekdays_compare_as_a_set_not_as_a_list() {
    let a = map_cron_to_schtasks("0 9 * * 5,1").unwrap();
    let b = map_cron_to_schtasks("0 9 * * 1,5").unwrap();
    assert_ne!(a, b, "the /D lists genuinely differ in order");
    assert_eq!(schtasks_when(&a), schtasks_when(&b));
}
