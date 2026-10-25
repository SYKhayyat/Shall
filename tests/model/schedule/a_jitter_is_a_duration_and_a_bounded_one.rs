use super::*;

#[test]
fn a_jitter_is_a_duration_and_a_bounded_one() {
    for (written, want) in [
        ("900", 900u32),
        ("15m", 900),
        ("2h", 7200),
        ("45s", 45),
        ("24h", 86400),
    ] {
        let o = opts(&[("cron", "0 2 * * *"), ("run", "clean"), ("jitter", written)]);
        let cfg = schedule_config("t", &o, &origin(), &shipped()).unwrap();
        assert_eq!(cfg.jitter, Some(want), "`jitter = {}` read wrong", written);
    }
    // Longer than a day, and an overflow that would wrap to a small number if it were
    // multiplied without asking.
    for written in ["25h", "2d", "soon", "-5", "4294967295h", ""] {
        let o = opts(&[("cron", "0 2 * * *"), ("run", "clean"), ("jitter", written)]);
        assert!(
            schedule_config("t", &o, &origin(), &shipped()).is_err(),
            "`jitter = {}` was accepted",
            written
        );
    }
}
