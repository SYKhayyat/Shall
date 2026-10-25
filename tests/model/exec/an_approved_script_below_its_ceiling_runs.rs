use super::*;

#[test]
fn an_approved_script_below_its_ceiling_runs() {
    let d = Decision::of(&Verdict::Approved, 0, Ceiling::read(None));
    assert_eq!(d, Decision::Run);
    assert!(d.will_run());
}
