use super::*;

#[test]
fn rpm_install_is_upgrade_or_install_not_bare_i() {
    // `rpm -i` refuses an already-installed package; the installing backend owns the
    // upgrade, so the argv must be the one that both installs and replaces.
    let argv = install_argv(Format::Rpm, &rpm()).unwrap();
    assert_eq!(argv[0], "rpm");
    assert_eq!(argv[1], "-U");
    assert!(argv.contains(&"/tmp/fd-10.2.0.x86_64.rpm".to_string()));
}
