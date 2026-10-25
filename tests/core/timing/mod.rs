pub use shall::core::timing::*;
pub use std::sync::atomic::{AtomicBool, Ordering};
pub use std::time::{Duration, Instant};

fn calls_of(rows: &[Row]) -> usize {
    rows.iter().map(|r| r.calls).sum()
}

fn end_records_the_time_that_actually_passed() {
    let started = begin();
    assert!(started.is_some(), "recording is on");
    std::thread::sleep(Duration::from_millis(30));
    end(started, "shall-timing-probe", &["sleep".to_string()]);
    let (rows, _, _) = summary();
    let row = rows
        .iter()
        .find(|r| r.label == "shall-timing-probe sleep")
        .expect("the span was recorded");
    assert!(
        row.total >= Duration::from_millis(10),
        "a 30ms child was recorded as {:?}",
        row.total
    );
}
mod a_run_is_gathered_per_label_slowest_first_and_off_by_default;
mod the_rendered_report_names_every_child_and_when_it_started;
