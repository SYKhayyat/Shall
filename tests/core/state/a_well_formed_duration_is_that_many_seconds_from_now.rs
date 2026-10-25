use super::*;

#[test]
fn a_well_formed_duration_is_that_many_seconds_from_now() {
    let now = StateRegistry::now();
    for (spec, secs) in [("30s", 30), ("5m", 300), ("2h", 7200), ("7d", 604_800)] {
        let at = StateRegistry::parse_duration(spec).expect(spec);
        assert!(
            at.abs_diff(now + secs) <= 2,
            "`{}` should be {}s out, got {}",
            spec,
            secs,
            at as i64 - now as i64
        );
    }
}
