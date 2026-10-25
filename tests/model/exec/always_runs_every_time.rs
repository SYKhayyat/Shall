use super::*;

#[test]
fn always_runs_every_time() {
    assert!(Decision::of(&Verdict::Approved, 500, Ceiling::read(Some("always"))).will_run());
}
