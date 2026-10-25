use super::*;

/// The element that opens the block is not one of the things inside it, and a closing tag is
/// not an opening one. Both would make "exactly one trigger" mean nothing.
#[test]
fn the_trigger_count_counts_triggers_and_not_the_block_around_them() {
    assert_eq!(trigger_count("<Triggers></Triggers>"), 0);
    assert_eq!(trigger_count("<Triggers><BootTrigger /></Triggers>"), 1);
    assert_eq!(
        trigger_count("<Triggers><BootTrigger><Delay>PT1M</Delay></BootTrigger></Triggers>"),
        1
    );
    assert_eq!(
        trigger_count("<Triggers><CalendarTrigger id=\"a\" /><EventTrigger /></Triggers>"),
        2
    );
}
