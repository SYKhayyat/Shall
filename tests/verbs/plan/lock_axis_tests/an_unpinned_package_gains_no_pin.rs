use super::*;

/// An upgrade is not a `lock`. A package nobody pinned has no stale record to fight, so it
/// gains no pin here — otherwise every `upgrade` would silently pin the whole machine.
#[test]
fn an_unpinned_package_gains_no_pin() {
    let mut locks = pins(&[("apt:curl", "7.81.0")]);
    let moved = move_pins_to(
        &mut locks,
        &pins(&[("apt:curl", "7.81.0"), ("cargo:ripgrep", "14.1.0")]),
    );
    assert_eq!(moved, 0, "nothing moved");
    assert_eq!(
        locks.len(),
        1,
        "an unpinned package was pinned: {:?}",
        locks
    );
}
