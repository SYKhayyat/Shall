use super::*;

#[test]
fn flatpak_without_a_channel_installs_the_bare_name() {
    let spec = spec_with("org.gimp.GIMP", &[]);
    assert_eq!(install_ref(&spec), "org.gimp.GIMP");
}
