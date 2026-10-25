use super::*;

#[test]
fn the_guard_key_is_what_opens_it() {
    // `[guard] confine_bin = false` restores the unchecked join. The opening is the
    // user's to make, and this asserts it is still there to make.
    assert!(bin_destination(&dir(), "../../.bashrc", false).is_ok());
}
