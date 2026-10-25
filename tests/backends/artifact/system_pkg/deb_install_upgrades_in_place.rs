use super::*;

#[test]
fn deb_install_upgrades_in_place() {
    assert_eq!(
        install_argv(Format::Deb, &deb()).unwrap(),
        vec!["dpkg", "-i", "/tmp/fd_10.2.0_amd64.deb"]
    );
}
