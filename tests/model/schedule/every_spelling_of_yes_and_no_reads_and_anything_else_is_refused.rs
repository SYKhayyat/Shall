use super::*;

#[test]
fn every_spelling_of_yes_and_no_reads_and_anything_else_is_refused() {
    for (written, want) in [
        ("true", true),
        ("yes", true),
        ("on", true),
        ("1", true),
        ("TRUE", true),
        ("false", false),
        ("no", false),
        ("off", false),
        ("0", false),
    ] {
        // Every boolean option, not just the one that was thought of: three keys go through
        // the same function and a check on one of them says nothing about the others.
        for key in ["enabled", "persistent", "elevated"] {
            let o = opts(&[("cron", "0 2 * * *"), ("run", "clean"), (key, written)]);
            let cfg = schedule_config("t", &o, &origin(), &shipped()).unwrap();
            let got = match key {
                "enabled" => cfg.enabled,
                "persistent" => cfg.persistent,
                _ => cfg.elevated,
            };
            assert_eq!(got, Some(want), "`{} = {}` read wrong", key, written);
        }
    }
    for key in ["enabled", "persistent", "elevated"] {
        let o = opts(&[("cron", "0 2 * * *"), ("run", "clean"), (key, "maybe")]);
        let err = schedule_config("t", &o, &origin(), &shipped()).unwrap_err();
        assert!(err.what.contains("not yes or no"), "{}", err);
        assert!(err.to_string().contains("schedules:3"), "{}", err);
    }
}
