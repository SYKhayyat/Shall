use super::*;

/// `runs=0` would mean "never runs", which is what deleting the line means. A ceiling that
/// silently disables the statement is the kind of quiet no-op II.2 refuses.
#[test]
fn a_zero_or_garbage_ceiling_is_refused() {
    for bad in [
        "exec:./s.sh@runs=0",
        "exec:./s.sh@runs=lots",
        "exec:./s.sh@runs=-1",
    ] {
        assert!(pv(bad).is_err(), "{} was accepted", bad);
    }
}
