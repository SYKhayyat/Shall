use super::*;

/// Re-recording twice in a row moves nothing the second time, so a `sync --upgrade` that
/// changed nothing does not rewrite the lockfile and make every run a commit.
#[test]
fn re_recording_an_already_current_pin_is_not_a_change() {
    let mut locks = pins(&[("apt:curl", "8.0.1")]);
    assert_eq!(move_pins_to(&mut locks, &pins(&[("apt:curl", "8.0.1")])), 0);
}
