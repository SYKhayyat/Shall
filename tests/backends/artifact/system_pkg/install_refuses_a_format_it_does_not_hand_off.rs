use super::*;

#[test]
fn install_refuses_a_format_it_does_not_hand_off() {
    assert!(install_argv(Format::Tarball, &deb()).is_err());
}
