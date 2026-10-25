use super::*;

#[test]
fn flatpak_channel_becomes_the_branch_of_the_ref() {
    let spec = spec_with("org.gimp.GIMP", &[("channel", "beta")]);
    assert_eq!(install_ref(&spec), "org.gimp.GIMP//beta");
}
