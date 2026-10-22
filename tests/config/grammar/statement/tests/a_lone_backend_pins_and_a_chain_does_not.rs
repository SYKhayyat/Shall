use super::*;

#[test]
fn a_lone_backend_pins_and_a_chain_does_not() {
    // The distinction the whole design rests on: `apt:rg` is apt or nothing, so it is
    // still apt on a machine that also has cargo. Anything with a comma is a preference,
    // not a pin, and the machine gets to answer.
    assert_eq!(
        cands("apt:curl"),
        (Some("apt".into()), Candidates::Priority)
    );
    assert_eq!(
        cands("apt,cargo:ripgrep"),
        (None, Candidates::Named(vec!["apt".into(), "cargo".into()]))
    );
}
