use super::*;

/// A manager that could not be read is not evidence that its package moved (V.7c's rule,
/// applied to the pins): the recorded version stays rather than being dropped or blanked.
#[test]
fn a_pin_the_scan_could_not_read_keeps_its_version() {
    let mut locks = pins(&[("apt:curl", "7.81.0"), ("brew:jq", "1.7")]);
    let moved = move_pins_to(&mut locks, &pins(&[("apt:curl", "8.0.1")]));
    assert_eq!(moved, 1);
    assert_eq!(locks["brew:jq"], Value::String("1.7".into()));
}
