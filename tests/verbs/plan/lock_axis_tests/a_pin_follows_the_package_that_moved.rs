use super::*;

/// Z2's second half: after an upgrade the pin names the version that was replaced, and the
/// next ordinary sync converges back down to it. Moving the pin is what stops that.
#[test]
fn a_pin_follows_the_package_that_moved() {
    let mut locks = pins(&[("apt:curl", "7.81.0")]);
    let moved = move_pins_to(&mut locks, &pins(&[("apt:curl", "8.0.1")]));
    assert_eq!(moved, 1);
    assert_eq!(locks["apt:curl"], Value::String("8.0.1".into()));
}
