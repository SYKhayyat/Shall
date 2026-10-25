use super::*;

#[test]
fn the_refusal_reads_the_command_not_the_whole_line() {
    // `run = sync --rebuild-cache` is not `run = rebuild`.
    let o = opts(&[("cron", "0 2 * * *"), ("run", "sync --locked")]);
    assert!(schedule_config("t", &o, &origin(), &shipped()).is_ok());
}
